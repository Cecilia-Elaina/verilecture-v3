use crate::{
    db::{CorrectionRule, ExamPoint, ImportedRecord, LexiconProfile, TranscriptSegment},
    ImportRequest,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

const MAX_AUDIO_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_AUDIO_DURATION_MS: i64 = 4 * 60 * 60 * 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JobControl {
    Running,
    Cancelled,
}

static JOB_CONTROLS: OnceLock<Mutex<HashMap<String, JobControl>>> = OnceLock::new();

fn job_controls() -> &'static Mutex<HashMap<String, JobControl>> {
    JOB_CONTROLS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn begin_job(job_id: &str) -> Result<(), String> {
    let mut controls = job_controls()
        .lock()
        .map_err(|_| "JOB_ALREADY_RUNNING".to_string())?;
    if controls.contains_key(job_id) {
        return Err("JOB_ALREADY_RUNNING".to_string());
    }
    controls.insert(job_id.to_string(), JobControl::Running);
    Ok(())
}

pub fn finish_job(job_id: &str) {
    if let Ok(mut controls) = job_controls().lock() {
        controls.remove(job_id);
    }
}

pub fn cancel_job(job_id: &str) -> Result<(), String> {
    let mut controls = job_controls()
        .lock()
        .map_err(|_| "JOB_CANCELLED".to_string())?;
    if !controls.contains_key(job_id) {
        return Err("JOB_CANCELLED".to_string());
    }
    controls.insert(job_id.to_string(), JobControl::Cancelled);
    Ok(())
}

fn check_job(job_id: &str) -> Result<(), String> {
    let state = job_controls()
        .lock()
        .map_err(|_| "JOB_CANCELLED".to_string())?
        .get(job_id)
        .copied()
        .unwrap_or(JobControl::Running);
    if state == JobControl::Cancelled {
        Err("JOB_CANCELLED".to_string())
    } else {
        Ok(())
    }
}

fn check_job_with_cleanup(job_id: &str, record_dir: &Path) -> Result<(), String> {
    match check_job(job_id) {
        Ok(()) => Ok(()),
        Err(error) => {
            if error == "JOB_CANCELLED" {
                let _ = std::fs::remove_dir_all(record_dir);
            }
            Err(error)
        }
    }
}

fn emit_progress(app: &AppHandle, job_id: &str, stage: &str, progress: u8, message: &str) {
    let _ = app.emit(
        "audio-job-progress",
        serde_json::json!({
            "jobId": job_id,
            "stage": stage,
            "progressPercent": progress,
            "message": message
        }),
    );
}

pub fn import_audio(
    app: &AppHandle,
    data_dir: &Path,
    model_dir: &Path,
    request: &ImportRequest,
    model_id: &str,
    provider_name: Option<&str>,
    lexicon: Option<&LexiconProfile>,
) -> Result<ImportedRecord, String> {
    let source = PathBuf::from(&request.path);
    if !source.is_file() {
        return Err("AUDIO_FILE_NOT_FOUND".to_string());
    }
    let source_size = std::fs::metadata(&source)
        .map_err(|_| "AUDIO_FILE_NOT_FOUND".to_string())?
        .len();
    if source_size > MAX_AUDIO_BYTES {
        return Err("AUDIO_FILE_TOO_LARGE".to_string());
    }
    let record_id = request
        .job_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    check_job(&record_id)?;
    emit_progress(app, &record_id, "copying", 5, "AUDIO_COPYING");
    let record_dir = data_dir.join("audio").join(&record_id);
    std::fs::create_dir_all(&record_dir).map_err(|_| "AUDIO_FILE_COPY_FAILED".to_string())?;
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("audio");
    let copied = record_dir.join(format!("source.{extension}"));
    std::fs::copy(&source, &copied).map_err(|_| "AUDIO_FILE_COPY_FAILED".to_string())?;
    check_job_with_cleanup(&record_id, &record_dir)?;
    emit_progress(app, &record_id, "decoding", 12, "AUDIO_DECODING");
    let samples = decode_to_pcm(&copied)?;
    let duration_ms = (samples.len() as i64 * 1000 / 16_000).max(1);
    if duration_ms > MAX_AUDIO_DURATION_MS {
        return Err("AUDIO_DURATION_EXCEEDED".to_string());
    }
    check_job_with_cleanup(&record_id, &record_dir)?;
    emit_progress(app, &record_id, "vad", 24, "AUDIO_VAD");
    let ranges = detect_speech_ranges(&samples, 16_000);
    if ranges.is_empty() {
        return Err("NO_SPEECH_DETECTED".to_string());
    }
    check_job_with_cleanup(&record_id, &record_dir)?;
    emit_progress(app, &record_id, "transcribing", 30, "AUDIO_TRANSCRIBING");
    let mut raw_segments = match run_sidecar(
        app,
        &record_id,
        data_dir,
        model_dir,
        model_id,
        &samples,
        &ranges,
        &request.language,
    ) {
        Ok(segments) => segments,
        Err(error) => {
            if error == "JOB_CANCELLED" {
                let _ = std::fs::remove_dir_all(&record_dir);
            }
            return Err(error);
        }
    };
    if raw_segments.is_empty() {
        return Err("ASR_MODEL_SMOKE_TEST_FAILED".to_string());
    }
    emit_progress(app, &record_id, "calibrating", 90, "AUDIO_CALIBRATING");
    check_job_with_cleanup(&record_id, &record_dir)?;
    let calibrated_segments = raw_segments
        .iter()
        .map(|segment| TranscriptSegment {
            id: format!("cal-{}", segment.id),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            text: calibrate_text(&segment.text, lexicon),
            language: segment.language.clone(),
            source: "calibrated".to_string(),
        })
        .collect::<Vec<_>>();
    emit_progress(app, &record_id, "completed", 100, "AUDIO_IMPORT_COMPLETED");
    finish_job(&record_id);
    Ok(ImportedRecord {
        id: record_id,
        title: request.title.clone(),
        source_path: Some(source.to_string_lossy().to_string()),
        audio_path: Some(copied.to_string_lossy().to_string()),
        created_at: Utc::now().to_rfc3339(),
        duration_ms,
        status: "completed".to_string(),
        model_id: model_id.to_string(),
        provider_name: provider_name.map(ToString::to_string),
        lexicon_id: request.lexicon_id.clone(),
        lexicon_version: lexicon.map(|value| value.version),
        language: request.language.clone(),
        raw_segments: std::mem::take(&mut raw_segments),
        calibrated_segments,
        exam_points: Vec::<ExamPoint>::new(),
    })
}

fn decode_to_pcm(path: &Path) -> Result<Vec<f32>, String> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("wav"))
        .unwrap_or(false)
    {
        return decode_wav(path);
    }
    let ffmpeg = locate_ffmpeg().ok_or_else(|| "AUDIO_DECODER_MISSING".to_string())?;
    let mut command = Command::new(ffmpeg);
    crate::runtime::configure_child_command(&mut command);
    let output = command
        .args(["-v", "error", "-i"])
        .arg(path)
        .args(["-ac", "1", "-ar", "16000", "-f", "f32le", "pipe:1"])
        .output()
        .map_err(|_| "AUDIO_DECODE_FAILED".to_string())?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err("AUDIO_DECODE_FAILED".to_string());
    }
    Ok(bytes_to_f32(&output.stdout))
}

fn decode_wav(path: &Path) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(path).map_err(|_| "AUDIO_DECODE_FAILED".to_string())?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("AUDIO_FORMAT_UNSUPPORTED".to_string());
    }
    let mut offset = 12;
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut bits = 0u16;
    let mut format = 0u16;
    let mut data: &[u8] = &[];
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let end = offset
            .saturating_add(8)
            .saturating_add(size)
            .min(bytes.len());
        if id == b"fmt " && end >= offset + 24 {
            format = u16::from_le_bytes(bytes[offset + 8..offset + 10].try_into().unwrap());
            channels = u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().unwrap());
            sample_rate = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
            bits = u16::from_le_bytes(bytes[offset + 22..offset + 24].try_into().unwrap());
        }
        if id == b"data" {
            data = &bytes[offset + 8..end];
        }
        offset = end + (size % 2);
    }
    if data.is_empty() || channels == 0 || sample_rate == 0 || ![1, 3].contains(&format) {
        return Err("AUDIO_FORMAT_UNSUPPORTED".to_string());
    }
    let mut mono = Vec::new();
    if format == 1 && bits == 16 {
        for frame in data.chunks_exact(channels as usize * 2) {
            let mut sum = 0.0;
            for channel in 0..channels as usize {
                let start = channel * 2;
                sum += i16::from_le_bytes([frame[start], frame[start + 1]]) as f32 / 32768.0;
            }
            mono.push(sum / channels as f32);
        }
    } else if format == 3 && bits == 32 {
        for frame in data.chunks_exact(channels as usize * 4) {
            let mut sum = 0.0;
            for channel in 0..channels as usize {
                let start = channel * 4;
                sum += f32::from_le_bytes(frame[start..start + 4].try_into().unwrap());
            }
            mono.push(sum / channels as f32);
        }
    } else {
        return Err("AUDIO_FORMAT_UNSUPPORTED".to_string());
    }
    if sample_rate == 16_000 {
        return Ok(mono);
    }
    resample_linear(&mono, sample_rate, 16_000)
}

fn resample_linear(input: &[f32], from: u32, to: u32) -> Result<Vec<f32>, String> {
    if input.is_empty() || from == 0 || to == 0 {
        return Err("AUDIO_RESAMPLE_FAILED".to_string());
    }
    let output_len = ((input.len() as u64 * to as u64) / from as u64).max(1) as usize;
    let ratio = from as f64 / to as f64;
    Ok((0..output_len)
        .map(|index| {
            let position = index as f64 * ratio;
            let left = position.floor() as usize;
            let fraction = (position - left as f64) as f32;
            let a = input[left.min(input.len() - 1)];
            let b = input[(left + 1).min(input.len() - 1)];
            a + (b - a) * fraction
        })
        .collect())
}

fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .filter(|value| value.is_finite())
        .collect()
}

fn detect_speech_ranges(samples: &[f32], sample_rate: usize) -> Vec<(usize, usize)> {
    let frame = sample_rate / 50;
    let mut ranges = Vec::new();
    let mut start: Option<usize> = None;
    let mut silence_frames = 0usize;
    for (index, chunk) in samples.chunks(frame).enumerate() {
        let rms = (chunk.iter().map(|value| value * value).sum::<f32>()
            / chunk.len().max(1) as f32)
            .sqrt();
        if rms > 0.008 {
            if start.is_none() {
                start = Some(index * frame);
            }
            silence_frames = 0;
        } else if let Some(begin) = start {
            silence_frames += 1;
            if silence_frames >= 40 {
                let end = (index.saturating_sub(39) * frame).min(samples.len());
                if end.saturating_sub(begin) >= sample_rate / 4 {
                    ranges.push((begin, end));
                }
                start = None;
                silence_frames = 0;
            }
        }
    }
    if let Some(begin) = start {
        if samples.len().saturating_sub(begin) >= sample_rate / 4 {
            ranges.push((begin, samples.len()));
        }
    }
    if ranges.is_empty() && !samples.is_empty() {
        let rms =
            (samples.iter().map(|value| value * value).sum::<f32>() / samples.len() as f32).sqrt();
        if rms > 0.002 {
            ranges.push((0, samples.len()));
        }
    }
    ranges
}

fn run_sidecar(
    app: &AppHandle,
    job_id: &str,
    data_dir: &Path,
    model_dir: &Path,
    model_id: &str,
    samples: &[f32],
    ranges: &[(usize, usize)],
    language: &str,
) -> Result<Vec<TranscriptSegment>, String> {
    let script =
        locate_sidecar(data_dir, model_id).ok_or_else(|| "MODEL_RUNTIME_MISSING".to_string())?;
    if !crate::runtime::manifest_is_valid_for_startup(&script) {
        return Err("MODEL_RUNTIME_CORRUPTED".to_string());
    }
    // A relative development-side sidecar must be made absolute before the
    // stable child working directory is applied.
    let script = std::fs::canonicalize(&script).unwrap_or(script);
    let model_path = model_dir.join(model_id);
    let mut sidecar = if is_sidecar_executable(&script) {
        Command::new(&script)
    } else {
        let python = locate_python().ok_or_else(|| "MODEL_PYTHON_RUNTIME_MISSING".to_string())?;
        let mut command = Command::new(python);
        command.arg(&script);
        command
    };
    crate::runtime::configure_child_command(&mut sidecar);
    let mut child = sidecar
        .args([
            "--protocol",
            "json-lines",
            "--model-id",
            model_id,
            "--model-dir",
        ])
        .arg(&model_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "MODEL_RUNTIME_FAILED".to_string())?;
    let mut requests = Vec::new();
    let mut transcribe_offsets_ms = Vec::new();
    requests.push(serde_json::json!({ "operation": "load", "requestId": Uuid::new_v4().to_string(), "modelId": model_id, "modelDir": model_path, "device": if model_id == "fun-asr-nano-2512" { "CPU" } else { "CUDA" } }).to_string());
    for (index, (start, end)) in ranges.iter().enumerate() {
        let mut chunk = *start;
        while chunk < *end {
            let chunk_end = (*end).min(chunk + 20 * 16_000);
            let raw = samples[chunk..chunk_end]
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<_>>();
            transcribe_offsets_ms.push(chunk as i64 * 1000 / 16_000);
            requests.push(serde_json::json!({ "operation": "transcribe", "requestId": Uuid::new_v4().to_string(), "modelId": model_id, "modelDir": model_path, "language": language, "audioPcmF32LeBase64": STANDARD.encode(raw), "chunkIndex": index, "startMs": chunk as i64 * 1000 / 16_000 }).to_string());
            chunk = chunk_end;
        }
    }
    requests.push(
        serde_json::json!({ "operation": "unload", "requestId": Uuid::new_v4().to_string() })
            .to_string(),
    );
    for request in requests {
        if let Err(error) = check_job(job_id) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        stdin
            .write_all(request.as_bytes())
            .map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
        stdin
            .write_all(b"\n")
            .map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
    }
    drop(stdin);
    let stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| "MODEL_RUNTIME_FAILED".to_string())?;
    let mut stdout_reader = std::io::BufReader::new(stdout_pipe);
    let total_transcribes = transcribe_offsets_ms.len().max(1);
    let mut segments = Vec::new();
    let mut line = String::new();
    let mut response_index = 0usize;
    loop {
        if let Err(error) = check_job(job_id) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        line.clear();
        let read = std::io::BufRead::read_line(&mut stdout_reader, &mut line)
            .map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
        if read == 0 {
            break;
        }
        let value: serde_json::Value =
            serde_json::from_str(line.trim()).map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
        if value.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
            return Err(value
                .get("errorCode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("MODEL_RUNTIME_FAILED")
                .to_string());
        }
        let offset_ms = response_index
            .checked_sub(1)
            .and_then(|index| transcribe_offsets_ms.get(index))
            .copied()
            .unwrap_or(0);
        if response_index > 0 && response_index <= total_transcribes {
            let percent = 30 + ((response_index * 60) / total_transcribes).min(60) as u8;
            emit_progress(app, job_id, "transcribing", percent, "AUDIO_TRANSCRIBING");
        }
        if let Some(items) = value.get("segments").and_then(serde_json::Value::as_array) {
            for item in items {
                let relative_start_ms = item
                    .get("startMs")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                let relative_end_ms = item
                    .get("endMs")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(relative_start_ms + 1);
                let start_ms = offset_ms + relative_start_ms;
                let end_ms = (offset_ms + relative_end_ms).max(start_ms + 1);
                let text = item
                    .get("text")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !text.is_empty() {
                    segments.push(TranscriptSegment {
                        id: Uuid::new_v4().to_string(),
                        start_ms,
                        end_ms,
                        text,
                        language: item
                            .get("language")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(language)
                            .to_string(),
                        source: "raw".to_string(),
                    });
                }
            }
        }
        response_index += 1;
    }
    let status = child
        .wait()
        .map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
    if !status.success() {
        return Err("MODEL_RUNTIME_FAILED".to_string());
    }
    segments.sort_by_key(|segment| (segment.start_ms, segment.end_ms));
    Ok(segments)
}

fn locate_python() -> Option<PathBuf> {
    for variable in ["VERILECTURE_PYTHON", "VERILECTURE_DEV_PYTHON"] {
        if let Ok(value) = std::env::var(variable) {
            let path = PathBuf::from(value);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    let executable_parent = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let bundled = [
        executable_parent.join("resources/asr-runtime/python/python.exe"),
        executable_parent.join("asr-runtime/python/python.exe"),
    ];
    if let Some(path) = bundled.into_iter().find(|path| path.is_file()) {
        return Some(path);
    }
    if cfg!(debug_assertions) {
        return Some(PathBuf::from("python"));
    }
    None
}

fn locate_sidecar(data_dir: &Path, model_id: &str) -> Option<PathBuf> {
    if let Ok(value) = std::env::var("VERILECTURE_ASR_RUNTIME") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
    }
    if model_id != "fun-asr-nano-2512" {
        let cuda_runtime = data_dir
            .join("runtimes")
            .join("cuda-qwen-fun")
            .join("verilecture-asr-runtime.exe");
        if cuda_runtime.is_file() {
            return Some(cuda_runtime);
        }
    }
    let candidates = [
        PathBuf::from("src-tauri/resources/asr-runtime/verilecture-asr-runtime.exe"),
        PathBuf::from("tools/asr/verilecture_asr_runtime.py"),
        PathBuf::from("../tools/asr/verilecture_asr_runtime.py"),
        std::env::current_exe()
            .ok()?
            .parent()?
            .join("resources/asr-runtime/verilecture-asr-runtime.exe"),
        std::env::current_exe()
            .ok()?
            .parent()?
            .join("resources/asr-runtime/verilecture_asr_runtime.py"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

fn locate_ffmpeg() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("VERILECTURE_FFMPEG") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
    }
    let executable_parent = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let bundled = [
        executable_parent.join("resources/ffmpeg/ffmpeg.exe"),
        executable_parent.join("ffmpeg/ffmpeg.exe"),
    ];
    if let Some(path) = bundled.into_iter().find(|path| path.is_file()) {
        return Some(path);
    }
    if cfg!(debug_assertions) {
        return Some(PathBuf::from("ffmpeg"));
    }
    None
}

fn is_sidecar_executable(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

fn calibrate_text(text: &str, lexicon: Option<&LexiconProfile>) -> String {
    let Some(lexicon) = lexicon else {
        return text.to_string();
    };
    let mut rules = lexicon.correction_rules.clone();
    for term in &lexicon.terms {
        for error in &term.common_asr_errors {
            rules.push(CorrectionRule {
                id: format!("term-{}", term.id),
                original_text: error.clone(),
                corrected_text: term.canonical_term.clone(),
                enabled: true,
                created_by: "lexicon".to_string(),
            });
        }
        for alias in &term.aliases {
            rules.push(CorrectionRule {
                id: format!("alias-{}", term.id),
                original_text: alias.clone(),
                corrected_text: term.canonical_term.clone(),
                enabled: true,
                created_by: "lexicon".to_string(),
            });
        }
    }
    rules.sort_by_key(|rule| std::cmp::Reverse(rule.original_text.chars().count()));
    rules
        .into_iter()
        .filter(|rule| {
            rule.enabled
                && !rule.original_text.is_empty()
                && !changes_sensitive_tokens(&rule.original_text, &rule.corrected_text)
        })
        .fold(text.to_string(), |current, rule| {
            current.replace(&rule.original_text, &rule.corrected_text)
        })
}

fn changes_sensitive_tokens(from: &str, to: &str) -> bool {
    let digits = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_ascii_digit())
            .collect::<String>()
    };
    let negation = |value: &str| {
        value.contains('不')
            || value.contains('没')
            || value.contains("禁止")
            || value.to_ascii_lowercase().contains("not")
    };
    digits(from) != digits(to) || negation(from) != negation(to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resampler_returns_finite_mono_samples() {
        let result = resample_linear(&[0.0, 1.0, 0.0, -1.0], 8_000, 16_000).unwrap();
        assert_eq!(result.len(), 8);
        assert!(result.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn sensitive_numbers_and_negation_are_protected() {
        assert!(changes_sensitive_tokens("TCP 123", "TCP 124"));
        assert!(changes_sensitive_tokens("不是可靠传输", "可靠传输"));
        assert!(!changes_sensitive_tokens("tcp", "TCP"));
    }

    #[test]
    fn cancellation_removes_the_managed_audio_copy() {
        let job_id = format!("cancel-cleanup-{}", Uuid::new_v4());
        let record_dir = std::env::temp_dir().join(format!("verilecture-audio-{job_id}"));
        std::fs::create_dir_all(&record_dir).unwrap();
        std::fs::write(record_dir.join("source.wav"), b"partial").unwrap();
        begin_job(&job_id).unwrap();
        cancel_job(&job_id).unwrap();
        assert_eq!(
            check_job_with_cleanup(&job_id, &record_dir).unwrap_err(),
            "JOB_CANCELLED"
        );
        assert!(!record_dir.exists());
        finish_job(&job_id);
    }
}
