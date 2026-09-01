use crate::db::ModelInstallState;
use crate::hardware::HardwareProfile;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Emitter};

const GB: u64 = 1024 * 1024 * 1024;
const REGISTRY_VERSION: &str = "2026-07-31-qwen-fun-official";
pub const CUDA_RUNTIME_ID: &str = "cuda-qwen-fun";
const RUNTIME_REGISTRY_SCHEMA_VERSION: u64 = 1;
const RUNTIME_REGISTRY_OVERRIDE_ENV: &str = "VERILECTURE_RUNTIME_REGISTRY_OVERRIDE";
const EMBEDDED_RUNTIME_REGISTRY: &str = include_str!("../resources/runtime_registry.json");

pub(crate) fn asr_runtime_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "verilecture-asr-runtime.exe"
    } else {
        "verilecture-asr-runtime"
    }
}

fn fun_runtime_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "llama-funasr-cli.exe"
    } else {
        "llama-funasr-cli"
    }
}

pub(crate) fn python_executable_names() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &["python.exe"]
    } else {
        &["python3", "python"]
    }
}

fn native_asr_runtime_available() -> bool {
    // The current bundled Fun-ASR and CUDA sidecars are Windows x64 builds.
    // Other targets can compile and launch the desktop shell, but must not
    // advertise a local ASR tier until a native sidecar is published.
    cfg!(target_os = "windows") && cfg!(target_arch = "x86_64")
}

fn host_runtime_matches(runtime: &RuntimeEntry) -> bool {
    runtime.platform == std::env::consts::OS && runtime.architecture == std::env::consts::ARCH
}

fn platform_label() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "linux" => "Linux",
        "macos" => "macOS",
        _ => "当前平台",
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub runtime: String,
    pub download_bytes: u64,
    pub disk_bytes: u64,
    pub requires_cuda: bool,
    pub requires_aligner: bool,
    pub status: String,
    pub supported: bool,
    pub recommended: bool,
    pub reason: String,
}

struct ModelDefinition {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    runtime: &'static str,
    runtime_bundle_id: Option<&'static str>,
    requires_cuda: bool,
    requires_aligner: bool,
}

struct Artifact {
    file_name: &'static str,
    url: &'static str,
    sha256: &'static str,
    bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeRegistry {
    schema_version: u64,
    registry_version: String,
    default_channel: String,
    runtimes: Vec<RuntimeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeEntry {
    id: String,
    version: String,
    channel: String,
    platform: String,
    architecture: String,
    artifact_name: String,
    compressed_bytes: u64,
    installed_bytes: u64,
    sha256: String,
    cuda_version: String,
    models: Vec<String>,
    requirements: RuntimeRequirements,
    status: String,
    mirrors: Vec<RuntimeMirror>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeRequirements {
    nvidia: bool,
    min_vram_bytes: u64,
    min_ram_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct RuntimeMirror {
    id: String,
    priority: u32,
    url: String,
    status: String,
}

#[derive(Debug, Clone)]
struct RuntimeArtifact {
    file_name: String,
    urls: Vec<String>,
    sha256: String,
    bytes: u64,
}

fn runtime_registry_source() -> Result<String, String> {
    if cfg!(debug_assertions) {
        if let Ok(path) = std::env::var(RUNTIME_REGISTRY_OVERRIDE_ENV) {
            if !path.trim().is_empty() {
                return std::fs::read_to_string(path)
                    .map_err(|_| "MODEL_RUNTIME_REGISTRY_INVALID".to_string());
            }
        }
    }
    Ok(EMBEDDED_RUNTIME_REGISTRY.to_string())
}

fn runtime_registry() -> Result<RuntimeRegistry, String> {
    let source = runtime_registry_source()?;
    let registry = serde_json::from_str::<RuntimeRegistry>(&source)
        .map_err(|_| "MODEL_RUNTIME_REGISTRY_INVALID".to_string())?;
    validate_runtime_registry(&registry)?;
    Ok(registry)
}

fn validate_runtime_registry(registry: &RuntimeRegistry) -> Result<(), String> {
    if registry.schema_version != RUNTIME_REGISTRY_SCHEMA_VERSION
        || registry.registry_version.trim().is_empty()
        || !matches!(registry.default_channel.as_str(), "alpha" | "stable")
        || registry.runtimes.is_empty()
    {
        return Err("MODEL_RUNTIME_REGISTRY_INVALID".to_string());
    }
    let mut runtime_keys = HashSet::new();
    for runtime in &registry.runtimes {
        if runtime.id.trim().is_empty()
            || !runtime_keys.insert((
                runtime.id.clone(),
                runtime.platform.clone(),
                runtime.architecture.clone(),
            ))
            || runtime.version.trim().is_empty()
            || runtime.channel != registry.default_channel
            || !matches!(runtime.platform.as_str(), "windows" | "linux" | "macos")
            || !matches!(runtime.architecture.as_str(), "x86_64" | "aarch64")
            || runtime.artifact_name.trim().is_empty()
            || !runtime.artifact_name.ends_with(".zip")
            || runtime.artifact_name.contains('/')
            || runtime.artifact_name.contains('\\')
            || runtime.version.contains('/')
            || runtime.version.contains('\\')
            || runtime.version == "."
            || runtime.version == ".."
            || runtime.compressed_bytes == 0
            || runtime.installed_bytes == 0
            || runtime.sha256.len() != 64
            || runtime.sha256 != runtime.sha256.to_ascii_lowercase()
            || !runtime
                .sha256
                .chars()
                .all(|value| value.is_ascii_hexdigit())
            || runtime.cuda_version.trim().is_empty()
            || runtime.models.is_empty()
            || !runtime.requirements.nvidia
            || runtime.requirements.min_vram_bytes == 0
            || runtime.requirements.min_ram_bytes == 0
            || !matches!(
                runtime.status.as_str(),
                "published" | "pending-publication" | "disabled"
            )
        {
            return Err("MODEL_RUNTIME_REGISTRY_INVALID".to_string());
        }
        let mut model_ids = HashSet::new();
        if runtime
            .models
            .iter()
            .any(|model_id| model_id.trim().is_empty() || !model_ids.insert(model_id))
        {
            return Err("MODEL_RUNTIME_REGISTRY_INVALID".to_string());
        }
        let mut mirror_ids = HashSet::new();
        let mut published_mirrors = 0;
        for mirror in &runtime.mirrors {
            if mirror.id.trim().is_empty()
                || !mirror_ids.insert(mirror.id.clone())
                || mirror.url.trim().is_empty()
                || !(mirror.url.starts_with("http://") || mirror.url.starts_with("https://"))
                || !matches!(
                    mirror.status.as_str(),
                    "published" | "pending-publication" | "disabled"
                )
            {
                return Err("MODEL_RUNTIME_REGISTRY_INVALID".to_string());
            }
            if mirror.status == "published" {
                published_mirrors += 1;
            }
        }
        if runtime.status == "published" && published_mirrors == 0 {
            return Err("MODEL_RUNTIME_REGISTRY_INVALID".to_string());
        }
        if runtime.status == "pending-publication" && published_mirrors > 0 {
            return Err("MODEL_RUNTIME_REGISTRY_INVALID".to_string());
        }
    }
    Ok(())
}

fn runtime_entry_for_model(model_id: &str) -> Result<Option<RuntimeEntry>, String> {
    let Some(definition) = definition(model_id) else {
        return Err("MODEL_PROFILE_NOT_SELECTED".to_string());
    };
    let Some(runtime_id) = definition.runtime_bundle_id else {
        return Ok(None);
    };
    let registry = runtime_registry()?;
    let runtime = registry
        .runtimes
        .into_iter()
        .find(|runtime| {
            runtime.id == runtime_id
                && host_runtime_matches(runtime)
                && runtime.models.iter().any(|id| id == model_id)
        })
        .ok_or_else(|| "MODEL_RUNTIME_MISSING".to_string())?;
    Ok(Some(runtime))
}

fn runtime_entry_for_id(runtime_id: &str) -> Result<RuntimeEntry, String> {
    let registry = runtime_registry()?;
    registry
        .runtimes
        .into_iter()
        .find(|runtime| runtime.id == runtime_id && host_runtime_matches(runtime))
        .ok_or_else(|| "MODEL_RUNTIME_MISSING".to_string())
}

fn published_runtime_artifact(runtime: &RuntimeEntry) -> Result<RuntimeArtifact, String> {
    if runtime.status != "published" {
        return Err("MODEL_RUNTIME_SOURCE_UNAVAILABLE".to_string());
    }
    let mut mirrors = runtime
        .mirrors
        .iter()
        .filter(|mirror| mirror.status == "published")
        .collect::<Vec<_>>();
    mirrors.sort_by_key(|mirror| mirror.priority);
    if mirrors.is_empty() {
        return Err("MODEL_RUNTIME_SOURCE_UNAVAILABLE".to_string());
    }
    Ok(RuntimeArtifact {
        file_name: runtime.artifact_name.clone(),
        urls: mirrors.iter().map(|mirror| mirror.url.clone()).collect(),
        sha256: runtime.sha256.clone(),
        bytes: runtime.compressed_bytes,
    })
}

fn runtime_download_requirements(model_id: &str) -> (u64, u64) {
    runtime_entry_for_model(model_id)
        .ok()
        .flatten()
        .map(|runtime| (runtime.compressed_bytes, runtime.installed_bytes))
        .unwrap_or((0, 0))
}

// Every artifact is pinned to an immutable Hugging Face or GitHub release
// commit. The byte count and SHA-256 are checked before an artifact enters the
// installed model directory. Keep this table in sync with
// src-tauri/resources/model-registry.json and docs/MODEL_REGISTRY.md.
static QWEN_17_ARTIFACTS: &[Artifact] = &[
    Artifact { file_name: "chat_template.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/chat_template.json", sha256: "75a8cfca24f00de72d796fbfed6858fc9614ef3dabd8696684cc3bc03a9c58ff", bytes: 1161 },
    Artifact { file_name: "config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/config.json", sha256: "2e74a751548b8ad7d7526d29365ad8144c345d8b412b1152d25dc6698452712f", bytes: 6194 },
    Artifact { file_name: "generation_config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/generation_config.json", sha256: "1da527824d81e07118facff437e03f2e24a23311e3bdeb2368973fe77e5f275c", bytes: 142 },
    Artifact { file_name: "merges.txt", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/merges.txt", sha256: "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5", bytes: 1671853 },
    Artifact { file_name: "model-00001-of-00002.safetensors", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/model-00001-of-00002.safetensors", sha256: "a4cd1f1a04d90b757dc7f7dd26254e69a013b19e80efe590a83c6a3bde8608d6", bytes: 4220320824 },
    Artifact { file_name: "model-00002-of-00002.safetensors", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/model-00002-of-00002.safetensors", sha256: "6e0b9d9e09e2e0238e7ef3cc8a484ab387e91b90f1900bedf88bc92d7929ccfc", bytes: 478200688 },
    Artifact { file_name: "model.safetensors.index.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/model.safetensors.index.json", sha256: "f994739fe38e5210b9e3e8ce6c6307315e2ceac3cb630e7b7414d69dce520f60", bytes: 64821 },
    Artifact { file_name: "preprocessor_config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/preprocessor_config.json", sha256: "45e120a4eda2c20c5d7f2ea9354e63536bf35e27aa573fb7cdf78017b378770d", bytes: 330 },
    Artifact { file_name: "tokenizer_config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/tokenizer_config.json", sha256: "4942d005604266809309cabc9f4e9cb89ce855d59b14681fdc0e1cc62ea26c4c", bytes: 12487 },
    Artifact { file_name: "vocab.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-1.7B/resolve/7278e1e70fe206f11671096ffdd38061171dd6e/vocab.json", sha256: "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910", bytes: 2776833 },
];

static QWEN_06_ARTIFACTS: &[Artifact] = &[
    Artifact { file_name: "chat_template.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/chat_template.json", sha256: "75a8cfca24f00de72d796fbfed6858fc9614ef3dabd8696684cc3bc03a9c58ff", bytes: 1161 },
    Artifact { file_name: "config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/config.json", sha256: "76d3ae4601ce939830b2517f4a6cadb86cc51316c3900af6b020b051c21a478c", bytes: 6193 },
    Artifact { file_name: "generation_config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/generation_config.json", sha256: "1da527824d81e07118facff437e03f2e24a23311e3bdeb2368973fe77e5f275c", bytes: 142 },
    Artifact { file_name: "merges.txt", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/merges.txt", sha256: "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5", bytes: 1671853 },
    Artifact { file_name: "model.safetensors", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/model.safetensors", sha256: "79d6cbd4c98c7bbffe9db2edac07f56cd6637d0d5944b27f6c2b8353840323ea", bytes: 1876091704 },
    Artifact { file_name: "preprocessor_config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/preprocessor_config.json", sha256: "45e120a4eda2c20c5d7f2ea9354e63536bf35e27aa573fb7cdf78017b378770d", bytes: 330 },
    Artifact { file_name: "tokenizer_config.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/tokenizer_config.json", sha256: "4942d005604266809309cabc9f4e9cb89ce855d59b14681fdc0e1cc62ea26c4c", bytes: 12487 },
    Artifact { file_name: "vocab.json", url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/5eb144179a02acc5e5ba31e748d22b0cf3e303b0/vocab.json", sha256: "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910", bytes: 2776833 },
];

static ALIGNER_ARTIFACTS: &[Artifact] = &[
    Artifact { file_name: "forced-aligner/chat_template.json", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/chat_template.json", sha256: "75a8cfca24f00de72d796fbfed6858fc9614ef3dabd8696684cc3bc03a9c58ff", bytes: 1161 },
    Artifact { file_name: "forced-aligner/config.json", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/config.json", sha256: "d616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09", bytes: 5982 },
    Artifact { file_name: "forced-aligner/generation_config.json", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/generation_config.json", sha256: "948d089b23bca1d214e768d59c4438365665f52ec6d33678f4062206b3fbbb8c", bytes: 115 },
    Artifact { file_name: "forced-aligner/merges.txt", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/merges.txt", sha256: "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5", bytes: 1671853 },
    Artifact { file_name: "forced-aligner/model.safetensors", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/model.safetensors", sha256: "47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7", bytes: 1835544544 },
    Artifact { file_name: "forced-aligner/preprocessor_config.json", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/preprocessor_config.json", sha256: "45e120a4eda2c20c5d7f2ea9354e63536bf35e27aa573fb7cdf78017b378770d", bytes: 330 },
    Artifact { file_name: "forced-aligner/tokenizer_config.json", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/tokenizer_config.json", sha256: "3ab80063f8511deb9566e6ad438d17b7a6277fcffd52d92854112f19d36bd81c", bytes: 12666 },
    Artifact { file_name: "forced-aligner/vocab.json", url: "https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/resolve/c7cbfc2048c462b0d63a45797104fc9db3ad62b7/vocab.json", sha256: "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910", bytes: 2776833 },
];

static FUN_ARTIFACTS: &[Artifact] = &[
    Artifact { file_name: "funasr-encoder-f16.gguf", url: "https://huggingface.co/FunAudioLLM/Fun-ASR-Nano-GGUF/resolve/46e849502a867080d66d351b8dfb1018b607e509/funasr-encoder-f16.gguf", sha256: "f92f91d01a24fbed6c863495b2ee8c6a6788144a02858b75743f0946668de8a2", bytes: 469331008 },
    Artifact { file_name: "qwen3-0.6b-q8_0.gguf", url: "https://huggingface.co/FunAudioLLM/Fun-ASR-Nano-GGUF/resolve/46e849502a867080d66d351b8dfb1018b607e509/qwen3-0.6b-q8_0.gguf", sha256: "819f385dc0e035dccc3d9e7edaf6b7b044b8ba7ace63cbcbf84c7e397eecbf27", bytes: 804753280 },
    Artifact { file_name: "fsmn-vad.gguf", url: "https://huggingface.co/FunAudioLLM/fsmn-vad-GGUF/resolve/6840bae4c5c92ee8c04faaf4db23dd0105098d7f/fsmn-vad.gguf", sha256: "1270f2559c495f4e7b6e739541151027d360761a3fda43fc147034f5719f5479", bytes: 1720512 },
    Artifact { file_name: "runtime/funasr-llamacpp-windows-x64.zip", url: "https://github.com/QwenAudio/Fun-ASR/releases/download/runtime-llamacpp-v0.1.9/funasr-llamacpp-windows-x64.zip", sha256: "6767af74e42c8b928742e12d5995c139636d9482ea151cdbb51f1b7573667772", bytes: 4685477 },
];

static DEFINITIONS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "qwen3-asr-1.7b",
        name: "Qwen3-ASR-1.7B",
        description: "高质量本地识别，配套 Qwen3 Forced Aligner 时间戳",
        runtime: "NVIDIA CUDA",
        runtime_bundle_id: Some(CUDA_RUNTIME_ID),
        requires_cuda: true,
        requires_aligner: true,
    },
    ModelDefinition {
        id: "qwen3-asr-0.6b",
        name: "Qwen3-ASR-0.6B",
        description: "较低显存占用，配套 Qwen3 Forced Aligner 时间戳",
        runtime: "NVIDIA CUDA",
        runtime_bundle_id: Some(CUDA_RUNTIME_ID),
        requires_cuda: true,
        requires_aligner: true,
    },
    ModelDefinition {
        id: "fun-asr-nano-2512",
        name: "Fun-ASR-Nano-2512",
        description: "无可用 NVIDIA CUDA 时的 CPU 本地档位；使用 VAD 段级时间范围",
        runtime: "CPU (Windows x64)",
        runtime_bundle_id: None,
        requires_cuda: false,
        requires_aligner: false,
    },
];

// Rust does not have a const slice concatenation helper. These static buffers
// are materialized once at startup and then exposed as immutable slices.
fn all_artifacts(model_id: &str) -> &'static [Artifact] {
    static QWEN_17_ALL: OnceLock<Vec<Artifact>> = OnceLock::new();
    static QWEN_06_ALL: OnceLock<Vec<Artifact>> = OnceLock::new();
    match model_id {
        "qwen3-asr-1.7b" => QWEN_17_ALL.get_or_init(|| {
            QWEN_17_ARTIFACTS
                .iter()
                .chain(ALIGNER_ARTIFACTS.iter())
                .map(|artifact| Artifact {
                    file_name: artifact.file_name,
                    url: artifact.url,
                    sha256: artifact.sha256,
                    bytes: artifact.bytes,
                })
                .collect()
        }),
        "qwen3-asr-0.6b" => QWEN_06_ALL.get_or_init(|| {
            QWEN_06_ARTIFACTS
                .iter()
                .chain(ALIGNER_ARTIFACTS.iter())
                .map(|artifact| Artifact {
                    file_name: artifact.file_name,
                    url: artifact.url,
                    sha256: artifact.sha256,
                    bytes: artifact.bytes,
                })
                .collect()
        }),
        _ => FUN_ARTIFACTS,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadControl {
    Running,
    Paused,
    Cancelled,
}
static CONTROLS: OnceLock<Mutex<HashMap<String, DownloadControl>>> = OnceLock::new();

fn controls() -> &'static Mutex<HashMap<String, DownloadControl>> {
    CONTROLS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
pub fn model_options(profile: Option<&HardwareProfile>, model_dir: &Path) -> Vec<ModelOption> {
    model_options_with_states(profile, model_dir, &HashMap::new())
}

pub fn model_options_with_states(
    profile: Option<&HardwareProfile>,
    model_dir: &Path,
    states: &HashMap<String, ModelInstallState>,
) -> Vec<ModelOption> {
    let recommended = recommended_id(profile);
    DEFINITIONS
        .iter()
        .map(|definition| {
            let (supported, reason) = support_reason(definition, profile);
            let status = model_status(model_dir, definition.id, states.get(definition.id));
            let download_bytes = artifact_bytes(definition.id);
            let disk_bytes = estimated_disk_bytes(definition.id);
            ModelOption {
                id: definition.id.to_string(),
                name: definition.name.to_string(),
                description: definition.description.to_string(),
                runtime: definition.runtime.to_string(),
                download_bytes,
                disk_bytes,
                requires_cuda: definition.requires_cuda,
                requires_aligner: definition.requires_aligner,
                status: status.to_string(),
                supported,
                recommended: recommended == Some(definition.id),
                reason: if status == "error" || status == "corrupted" {
                    states
                        .get(definition.id)
                        .and_then(|state| state.error_code.clone())
                        .map(|error| format!("最近一次安装失败：{error}"))
                        .unwrap_or(reason)
                } else {
                    reason
                },
            }
        })
        .collect()
}

fn model_status(model_dir: &Path, model_id: &str, state: Option<&ModelInstallState>) -> String {
    if is_ready(model_dir, model_id) {
        return "ready".to_string();
    }
    if let Some(state) = state {
        return match state.stage.as_str() {
            "checking_disk" | "resolving" => "checking".to_string(),
            "downloading" | "downloading_runtime" | "downloading_model" => {
                "downloading".to_string()
            }
            "paused" => "paused".to_string(),
            "verifying" => "verifying".to_string(),
            "installing" => "installing".to_string(),
            "probing_runtime" | "loading_model" | "testing" | "smoke_test" => "testing".to_string(),
            "cancelled" => "cancelled".to_string(),
            "corrupted" => "corrupted".to_string(),
            "failed" => "error".to_string(),
            _ => "not_installed".to_string(),
        };
    }
    if model_dir.join(format!(".{model_id}.installing")).is_dir() {
        return "paused".to_string();
    }
    "not_installed".to_string()
}

fn artifact_bytes(model_id: &str) -> u64 {
    artifacts_for(model_id)
        .iter()
        .map(|artifact| artifact.bytes)
        .sum()
}

fn estimated_disk_bytes(model_id: &str) -> u64 {
    let artifact_total = artifact_bytes(model_id);
    let (runtime_download, runtime_installed) = runtime_download_requirements(model_id);
    artifact_total
        .saturating_add(runtime_download)
        .saturating_add(runtime_installed)
        .saturating_add(((artifact_total.saturating_add(runtime_installed)) as f64 * 0.20) as u64)
        .saturating_add(2 * GB)
}

pub fn cuda_runtime_directory(data_dir: &Path) -> PathBuf {
    runtime_entry_for_id(CUDA_RUNTIME_ID)
        .map(|runtime| runtime_directory_for(data_dir, &runtime))
        .unwrap_or_else(|_| data_dir.join("runtimes").join(CUDA_RUNTIME_ID))
}

fn runtime_directory_for(data_dir: &Path, runtime: &RuntimeEntry) -> PathBuf {
    data_dir
        .join("runtimes")
        .join(&runtime.id)
        .join(&runtime.version)
}

fn cuda_runtime_staging_directory(data_dir: &Path, runtime: &RuntimeEntry) -> PathBuf {
    data_dir
        .join("runtimes")
        .join(&runtime.id)
        .join(format!(".{}.installing", runtime.version))
}

fn cuda_runtime_is_ready(data_dir: &Path) -> bool {
    let Some(sidecar) = locate_cuda_sidecar(data_dir) else {
        return false;
    };
    crate::runtime::manifest_is_valid_for_startup(&sidecar) && probe_cuda_sidecar(&sidecar)
}

pub fn selected_model_id(database: &crate::db::AppDatabase) -> Result<Option<String>, String> {
    database.get_setting("selected_model_id")
}

pub fn model_states(
    database: &crate::db::AppDatabase,
) -> Result<HashMap<String, ModelInstallState>, String> {
    let mut states = HashMap::new();
    for definition in DEFINITIONS {
        if let Some(state) = database.model_install_state(definition.id)? {
            states.insert(definition.id.to_string(), state);
        }
    }
    Ok(states)
}

pub fn is_ready(model_dir: &Path, model_id: &str) -> bool {
    if definition(model_id).is_none() {
        return false;
    }
    let destination = model_dir.join(model_id);
    let marker = destination.join("READY.json");
    let Ok(bytes) = std::fs::read(marker) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    if value.get("modelId").and_then(serde_json::Value::as_str) != Some(model_id)
        || value
            .get("registryVersion")
            .and_then(serde_json::Value::as_str)
            != Some(REGISTRY_VERSION)
        || value.get("smokeTest").and_then(serde_json::Value::as_str) != Some("passed")
        || value
            .get("timestampTest")
            .and_then(serde_json::Value::as_str)
            != Some("passed")
    {
        return false;
    }
    let Some(signatures) = value
        .get("fileSignatures")
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    for artifact in artifacts_for(model_id)
        .iter()
        .filter(|artifact| !artifact.file_name.ends_with(".zip"))
    {
        let Some(signature) = signatures
            .get(artifact.file_name)
            .and_then(serde_json::Value::as_object)
        else {
            return false;
        };
        let Some(expected_modified_at) = signature
            .get("modifiedAtNs")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<u128>().ok())
        else {
            return false;
        };
        let Some((bytes, modified_at)) = file_signature(&destination.join(artifact.file_name))
        else {
            return false;
        };
        if bytes != artifact.bytes || modified_at != expected_modified_at {
            return false;
        }
    }
    if model_id == "fun-asr-nano-2512" && !find_fun_runtime(&destination) {
        return false;
    }
    true
}

pub fn verify_model_integrity(model_dir: &Path, model_id: &str) -> Result<(), String> {
    if definition(model_id).is_none() {
        return Err("MODEL_PROFILE_NOT_SELECTED".to_string());
    }
    let destination = model_dir.join(model_id);
    if !is_ready(model_dir, model_id) {
        return Err("MODEL_NOT_INSTALLED".to_string());
    }
    for artifact in artifacts_for(model_id) {
        if artifact.file_name.ends_with(".zip") {
            continue;
        }
        verify_sha256(
            &destination.join(artifact.file_name),
            artifact.sha256,
            artifact.bytes,
        )
        .map_err(|_| "MODEL_CORRUPTED".to_string())?;
    }
    if model_id == "fun-asr-nano-2512" && !find_fun_runtime(&destination) {
        return Err("MODEL_CORRUPTED".to_string());
    }
    refresh_ready_signatures(&destination, model_id)?;
    Ok(())
}

pub fn artifact_bytes_for_diagnostics(model_id: &str) -> u64 {
    artifact_bytes(model_id)
}

fn find_fun_runtime(directory: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            find_fun_runtime(&path)
        } else {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.eq_ignore_ascii_case(fun_runtime_executable_name()))
                .unwrap_or(false)
        }
    })
}

fn file_signature(path: &Path) -> Option<(u64, u128)> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let elapsed = modified.duration_since(UNIX_EPOCH).ok()?;
    Some((metadata.len(), elapsed.as_nanos()))
}

fn file_signatures(
    model_id: &str,
    directory: &Path,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let mut signatures = serde_json::Map::new();
    for artifact in artifacts_for(model_id)
        .iter()
        .filter(|artifact| !artifact.file_name.ends_with(".zip"))
    {
        let Some((bytes, modified_at)) = file_signature(&directory.join(artifact.file_name)) else {
            return Err("MODEL_CORRUPTED".to_string());
        };
        if bytes != artifact.bytes {
            return Err("MODEL_CHECKSUM_MISMATCH".to_string());
        }
        signatures.insert(
            artifact.file_name.to_string(),
            serde_json::json!({
                "bytes": bytes,
                "modifiedAtNs": modified_at.to_string(),
            }),
        );
    }
    Ok(signatures)
}

fn refresh_ready_signatures(directory: &Path, model_id: &str) -> Result<(), String> {
    let marker = directory.join("READY.json");
    let bytes = std::fs::read(&marker).map_err(|_| "MODEL_CORRUPTED".to_string())?;
    let mut value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "MODEL_CORRUPTED".to_string())?;
    let signatures = file_signatures(model_id, directory)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "MODEL_CORRUPTED".to_string())?;
    object.insert(
        "fileSignatures".to_string(),
        serde_json::Value::Object(signatures),
    );
    object.insert(
        "verifiedAt".to_string(),
        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
    );
    std::fs::write(
        marker,
        serde_json::to_vec_pretty(&value).map_err(|_| "MODEL_CORRUPTED".to_string())?,
    )
    .map_err(|_| "MODEL_CORRUPTED".to_string())
}

fn definition(model_id: &str) -> Option<&'static ModelDefinition> {
    DEFINITIONS
        .iter()
        .find(|definition| definition.id == model_id)
}

fn artifacts_for(model_id: &str) -> &'static [Artifact] {
    all_artifacts(model_id)
}

fn recommended_id(profile: Option<&HardwareProfile>) -> Option<&'static str> {
    if !native_asr_runtime_available() {
        return None;
    }
    let profile = profile?;
    if profile.nvidia_detected
        && profile.vram_bytes.unwrap_or(0) >= 8 * GB
        && profile.total_ram_bytes >= 16 * GB
    {
        Some("qwen3-asr-1.7b")
    } else if profile.nvidia_detected
        && profile.vram_bytes.unwrap_or(0) >= 6 * GB
        && profile.total_ram_bytes >= 16 * GB
    {
        Some("qwen3-asr-0.6b")
    } else {
        Some("fun-asr-nano-2512")
    }
}

fn support_reason(
    definition: &ModelDefinition,
    profile: Option<&HardwareProfile>,
) -> (bool, String) {
    if !native_asr_runtime_available() {
        return (
            false,
            format!("{} 构建暂未包含可用的本地 ASR 运行时", platform_label()),
        );
    }
    let Some(profile) = profile else {
        return (
            definition.id == "fun-asr-nano-2512",
            if definition.id == "fun-asr-nano-2512" {
                "等待完成硬件扫描后确认 CPU 档位".to_string()
            } else {
                "等待完成硬件扫描".to_string()
            },
        );
    };
    if !profile.model_directory_writable {
        return (false, "模型目录不可写".to_string());
    }
    if profile.disk_free_bytes < estimated_disk_bytes(definition.id) {
        return (false, "可用磁盘空间不足".to_string());
    }
    if definition.id == "qwen3-asr-1.7b" {
        if profile.total_ram_bytes < 16 * GB {
            return (false, "系统内存至少需要 16 GB".to_string());
        }
        if profile.vram_bytes.unwrap_or(0) < 8 * GB {
            return (false, "显存至少需要 8 GB".to_string());
        }
    }
    if definition.id == "qwen3-asr-0.6b" {
        if profile.total_ram_bytes < 16 * GB {
            return (false, "系统内存至少需要 16 GB".to_string());
        }
        if profile.vram_bytes.unwrap_or(0) < 6 * GB {
            return (false, "显存至少需要 6 GB".to_string());
        }
    }
    if definition.requires_cuda && !profile.cuda_smoke_test {
        if profile.nvidia_detected {
            return (
                true,
                "CUDA runtime will be downloaded and smoke-tested before Qwen installation."
                    .to_string(),
            );
        }
        return (false, "CUDA is unavailable on this computer.".to_string());
    }
    if definition.id == "fun-asr-nano-2512" && profile.total_ram_bytes < 8 * GB {
        return (false, "系统内存至少需要 8 GB".to_string());
    }
    (
        true,
        if definition.id == "fun-asr-nano-2512" {
            "支持 CPU 推理；速度通常较慢".to_string()
        } else {
            "满足静态硬件条件，安装时仍需通过真实模型 Smoke Test".to_string()
        },
    )
}

pub fn is_supported(model_id: &str, profile: Option<&HardwareProfile>) -> bool {
    definition(model_id)
        .map(|definition| support_reason(definition, profile).0)
        .unwrap_or(false)
}

pub async fn set_download_control(model_id: &str, action: &str) -> Result<(), String> {
    let control = match action {
        "pause" => DownloadControl::Paused,
        "resume" => DownloadControl::Running,
        "cancel" => DownloadControl::Cancelled,
        _ => return Err("MODEL_DOWNLOAD_CONTROL_INVALID".to_string()),
    };
    controls()
        .lock()
        .map_err(|_| "MODEL_DOWNLOAD_CONTROL_FAILED".to_string())?
        .insert(model_id.to_string(), control);
    Ok(())
}

pub async fn install_model(
    app: &AppHandle,
    database: &crate::db::AppDatabase,
    data_dir: &Path,
    model_dir: &Path,
    model_id: &str,
) -> Result<(), String> {
    if !native_asr_runtime_available() {
        return Err("MODEL_RUNTIME_UNAVAILABLE".to_string());
    }
    if definition(model_id).is_none() {
        return Err("MODEL_PROFILE_NOT_SELECTED".to_string());
    }
    let artifacts = artifacts_for(model_id);
    if artifacts.is_empty() {
        return Err("MODEL_MANIFEST_INVALID".to_string());
    }
    let runtime = runtime_entry_for_model(model_id)?;
    controls()
        .lock()
        .map_err(|_| "MODEL_DOWNLOAD_CONTROL_FAILED".to_string())?
        .insert(model_id.to_string(), DownloadControl::Running);
    let runtime_required = runtime.is_some() && !cuda_runtime_is_ready(data_dir);
    let runtime_bytes = runtime
        .as_ref()
        .filter(|_| runtime_required)
        .map(|runtime| runtime.compressed_bytes)
        .unwrap_or(0);
    let total_bytes = runtime_bytes.saturating_add(artifact_bytes(model_id));
    let _ =
        database.record_model_install_event(model_id, "checking_disk", None, 0, total_bytes, None);
    if model_dir
        .metadata()
        .ok()
        .and_then(|metadata| metadata.is_dir().then_some(()))
        .is_none()
        && std::fs::create_dir_all(model_dir).is_err()
    {
        let _ = database.record_model_install_event(
            model_id,
            "failed",
            None,
            0,
            total_bytes,
            Some("MODEL_DOWNLOAD_FAILED"),
        );
        return Err("MODEL_DOWNLOAD_FAILED".to_string());
    }
    if runtime_required {
        install_cuda_runtime(
            app,
            database,
            data_dir,
            model_id,
            runtime
                .as_ref()
                .ok_or_else(|| "MODEL_RUNTIME_MISSING".to_string())?,
            total_bytes,
        )
        .await?;
    }
    let staging = model_dir.join(format!(".{model_id}.installing"));
    std::fs::create_dir_all(&staging).map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?;
    let client = reqwest::Client::new();
    let result: Result<(), String> = async {
        let mut completed_bytes = runtime_bytes;
        for artifact in artifacts {
            wait_for_control(model_id).await?;
            if artifact_is_installed(artifact, &staging) {
                completed_bytes = completed_bytes.saturating_add(artifact.bytes);
                continue;
            }
            let part = staging.join(format!("{}.part", artifact.file_name));
            let final_path = staging.join(artifact.file_name);
            let _ = database.record_model_install_event(
                model_id,
                "downloading",
                Some(artifact.file_name),
                completed_bytes,
                total_bytes,
                None,
            );
            let result = download_artifact(
                Some(app),
                &client,
                model_id,
                artifact,
                &part,
                completed_bytes,
                total_bytes,
            )
            .await;
            if let Err(error) = result {
                if error == "MODEL_CHECKSUM_MISMATCH" {
                    let _ = std::fs::remove_file(&part);
                }
                return Err(error);
            }
            emit_stage(
                app,
                model_id,
                "verifying",
                artifact.file_name,
                completed_bytes.saturating_add(artifact.bytes),
                total_bytes,
                "Verifying checksum",
            );
            let _ = database.record_model_install_event(
                model_id,
                "verifying",
                Some(artifact.file_name),
                completed_bytes.saturating_add(artifact.bytes),
                total_bytes,
                None,
            );
            if let Err(error) = verify_sha256(&part, artifact.sha256, artifact.bytes) {
                let _ = std::fs::remove_file(&part);
                return Err(error);
            }
            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent).map_err(|_| "MODEL_INSTALL_FAILED".to_string())?;
            }
            std::fs::rename(&part, &final_path).map_err(|_| "MODEL_CORRUPTED".to_string())?;
            if artifact.file_name.ends_with(".zip") {
                extract_zip(&final_path, &staging)?;
                std::fs::remove_file(&final_path)
                    .map_err(|_| "MODEL_INSTALL_FAILED".to_string())?;
            }
            completed_bytes = completed_bytes.saturating_add(artifact.bytes);
        }
        emit_stage(
            app,
            model_id,
            "testing",
            "",
            completed_bytes,
            total_bytes,
            "Loading model and running smoke test",
        );
        let _ = database.record_model_install_event(
            model_id,
            "testing",
            None,
            completed_bytes,
            total_bytes,
            None,
        );
        run_smoke_test(model_id, &staging, data_dir)?;
        let signatures = file_signatures(model_id, &staging)?;
        let payload = serde_json::json!({
            "modelId": model_id,
            "registryVersion": REGISTRY_VERSION,
            "verifiedAt": chrono::Utc::now().to_rfc3339(),
            "smokeTest": "passed",
            "timestampTest": "passed",
            "fileSignatures": signatures,
            "artifacts": artifacts.iter().map(|artifact| serde_json::json!({
                "path": artifact.file_name,
                "bytes": artifact.bytes,
                "sha256": artifact.sha256,
            })).collect::<Vec<_>>()
        });
        std::fs::write(
            staging.join("READY.json"),
            serde_json::to_vec_pretty(&payload).map_err(|_| "MODEL_CORRUPTED".to_string())?,
        )
        .map_err(|_| "MODEL_CORRUPTED".to_string())?;
        Ok(())
    }
    .await;
    if let Err(error) = result {
        let stage = if error == "MODEL_DOWNLOAD_CANCELLED" {
            "cancelled"
        } else if matches!(
            error.as_str(),
            "MODEL_CHECKSUM_MISMATCH" | "MODEL_CORRUPTED" | "MODEL_MANIFEST_INVALID"
        ) {
            "corrupted"
        } else {
            "failed"
        };
        let _ = database.record_model_install_event(
            model_id,
            stage,
            None,
            0,
            total_bytes,
            Some(&error),
        );
        let _ = app.emit(
            "model-install-progress",
            serde_json::json!({
                "modelId": model_id,
                "stage": if stage == "cancelled" { "error" } else { "error" },
                "fileName": "",
                "downloadedBytes": 0,
                "totalBytes": total_bytes,
                "speedBytesPerSecond": 0,
                "message": error
            }),
        );
        return Err(error);
    }
    let destination = model_dir.join(model_id);
    let backup = model_dir.join(format!(".{model_id}.previous"));
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(|_| "MODEL_INSTALL_CLEANUP_FAILED".to_string())?;
    }
    if destination.exists() {
        std::fs::rename(&destination, &backup)
            .map_err(|_| "MODEL_INSTALL_SWAP_FAILED".to_string())?;
    }
    if let Err(error) = std::fs::rename(&staging, &destination) {
        if backup.exists() && !destination.exists() {
            let _ = std::fs::rename(&backup, &destination);
        }
        let message = format!("MODEL_INSTALL_SWAP_FAILED:{error}");
        let _ = database.record_model_install_event(
            model_id,
            "failed",
            None,
            total_bytes,
            total_bytes,
            Some(&message),
        );
        return Err(message);
    }
    emit_stage(
        app,
        model_id,
        "ready",
        "",
        total_bytes,
        total_bytes,
        "READY",
    );
    let _ = database.record_model_install_event(
        model_id,
        "ready",
        None,
        total_bytes,
        total_bytes,
        None,
    );
    let manifest = std::fs::read_to_string(destination.join("READY.json"))
        .map_err(|_| "MODEL_CORRUPTED".to_string())?;
    database.save_installed_model(
        model_id,
        REGISTRY_VERSION,
        &destination.to_string_lossy(),
        &manifest,
    )?;
    Ok(())
}

async fn install_cuda_runtime(
    app: &AppHandle,
    database: &crate::db::AppDatabase,
    data_dir: &Path,
    model_id: &str,
    runtime: &RuntimeEntry,
    total_bytes: u64,
) -> Result<(), String> {
    let runtime_artifact = match published_runtime_artifact(runtime) {
        Ok(artifact) => artifact,
        Err(error) => {
            let _ = database.record_model_install_event(
                model_id,
                "failed",
                Some(&runtime.artifact_name),
                0,
                total_bytes,
                Some(&error),
            );
            return Err(error);
        }
    };
    let runtime_root = data_dir.join("runtimes").join(&runtime.id);
    std::fs::create_dir_all(&runtime_root)
        .map_err(|_| "MODEL_RUNTIME_DOWNLOAD_FAILED".to_string())?;
    let staging = cuda_runtime_staging_directory(data_dir, runtime);
    std::fs::create_dir_all(&staging).map_err(|_| "MODEL_RUNTIME_DOWNLOAD_FAILED".to_string())?;
    let part = staging.join(format!("{}.part", runtime_artifact.file_name));
    let _ = database.record_model_install_event(
        model_id,
        "downloading_runtime",
        Some(&runtime_artifact.file_name),
        0,
        total_bytes,
        None,
    );
    let client = reqwest::Client::new();
    download_runtime_artifact(
        Some(app),
        &client,
        model_id,
        &runtime_artifact,
        &part,
        0,
        total_bytes,
    )
    .await
    .map_err(|error| {
        let _ = database.record_model_install_event(
            model_id,
            "failed",
            Some(&runtime_artifact.file_name),
            0,
            total_bytes,
            Some(&error),
        );
        error
    })?;
    emit_stage(
        app,
        model_id,
        "verifying",
        &runtime_artifact.file_name,
        runtime_artifact.bytes,
        total_bytes,
        "Verifying checksum",
    );
    verify_sha256(&part, &runtime_artifact.sha256, runtime_artifact.bytes)?;
    emit_stage(
        app,
        model_id,
        "installing",
        &runtime_artifact.file_name,
        runtime_artifact.bytes,
        total_bytes,
        "Installing CUDA runtime",
    );
    extract_zip(&part, &staging)?;
    std::fs::remove_file(&part).map_err(|_| "MODEL_RUNTIME_INSTALL_FAILED".to_string())?;
    let staged_sidecar = staging.join(asr_runtime_executable_name());
    if !crate::runtime::manifest_is_valid_for_startup(&staged_sidecar)
        || !probe_cuda_sidecar(&staged_sidecar)
    {
        return Err("CUDA_RUNTIME_SMOKE_TEST_FAILED".to_string());
    }
    let destination = runtime_directory_for(data_dir, runtime);
    let backup = runtime_root.join(format!(".{}.previous", runtime.version));
    if backup.exists() {
        std::fs::remove_dir_all(&backup)
            .map_err(|_| "MODEL_RUNTIME_INSTALL_CLEANUP_FAILED".to_string())?;
    }
    if destination.exists() {
        std::fs::rename(&destination, &backup)
            .map_err(|_| "MODEL_RUNTIME_INSTALL_SWAP_FAILED".to_string())?;
    }
    if let Err(error) = std::fs::rename(&staging, &destination) {
        if backup.exists() && !destination.exists() {
            let _ = std::fs::rename(&backup, &destination);
        }
        return Err(format!("MODEL_RUNTIME_INSTALL_SWAP_FAILED:{error}"));
    }
    emit_stage(
        app,
        model_id,
        "installing",
        &runtime_artifact.file_name,
        runtime_artifact.bytes,
        total_bytes,
        "CUDA runtime ready",
    );
    let _ = database.record_model_install_event(
        model_id,
        "installing",
        Some(&runtime_artifact.file_name),
        runtime_artifact.bytes,
        total_bytes,
        None,
    );
    let _ = std::fs::remove_dir_all(&backup);
    Ok(())
}

fn artifact_is_installed(artifact: &Artifact, staging: &Path) -> bool {
    if artifact.file_name.ends_with(".zip") {
        return find_fun_runtime(staging);
    }
    let path = staging.join(artifact.file_name);
    std::fs::metadata(&path)
        .map(|metadata| metadata.len() == artifact.bytes)
        .unwrap_or(false)
        && verify_sha256(&path, artifact.sha256, artifact.bytes).is_ok()
}

fn emit_stage(
    app: &AppHandle,
    model_id: &str,
    stage: &str,
    file_name: &str,
    downloaded: u64,
    total: u64,
    message: &str,
) {
    let _ = app.emit("model-install-progress", serde_json::json!({ "modelId": model_id, "stage": stage, "fileName": file_name, "downloadedBytes": downloaded, "totalBytes": total, "speedBytesPerSecond": 0, "message": message }));
}

fn emit_stage_optional(
    app: Option<&AppHandle>,
    model_id: &str,
    stage: &str,
    file_name: &str,
    downloaded: u64,
    total: u64,
    message: &str,
) {
    if let Some(app) = app {
        emit_stage(app, model_id, stage, file_name, downloaded, total, message);
    }
}

async fn wait_for_control(model_id: &str) -> Result<(), String> {
    loop {
        let state = controls()
            .lock()
            .map_err(|_| "MODEL_DOWNLOAD_CONTROL_FAILED".to_string())?
            .get(model_id)
            .copied()
            .unwrap_or(DownloadControl::Running);
        match state {
            DownloadControl::Running => return Ok(()),
            DownloadControl::Cancelled => return Err("MODEL_DOWNLOAD_CANCELLED".to_string()),
            DownloadControl::Paused => {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await
            }
        }
    }
}

fn artifact_urls(artifact: &Artifact) -> Vec<String> {
    let mut urls = Vec::with_capacity(2);
    if let Some(mirror) = modelscope_mirror_url(artifact.url) {
        urls.push(mirror);
    }
    urls.push(artifact.url.to_string());
    urls
}

fn modelscope_mirror_url(url: &str) -> Option<String> {
    let path = url.strip_prefix("https://huggingface.co/")?;
    let mut segments = path.split('/');
    let owner = segments.next()?;
    let repository = segments.next()?;
    if segments.next()? != "resolve" {
        return None;
    }
    let _revision = segments.next()?;
    let file_path = segments.collect::<Vec<_>>().join("/");
    if file_path.is_empty() {
        return None;
    }
    Some(format!(
        "https://modelscope.cn/models/{owner}/{repository}/resolve/master/{file_path}"
    ))
}

async fn download_artifact(
    app: Option<&AppHandle>,
    client: &reqwest::Client,
    model_id: &str,
    artifact: &Artifact,
    part: &Path,
    completed_bytes: u64,
    total_bytes: u64,
) -> Result<(), String> {
    if let Some(parent) = part.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?;
    }
    let existing = std::fs::metadata(part)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if existing == artifact.bytes {
        verify_sha256(part, artifact.sha256, artifact.bytes)?;
        return Ok(());
    }
    let urls = artifact_urls(artifact);
    download_artifact_from_sources(
        app,
        client,
        model_id,
        artifact.file_name,
        &urls,
        artifact.bytes,
        part,
        completed_bytes,
        total_bytes,
    )
    .await
}

async fn download_runtime_artifact(
    app: Option<&AppHandle>,
    client: &reqwest::Client,
    model_id: &str,
    artifact: &RuntimeArtifact,
    part: &Path,
    completed_bytes: u64,
    total_bytes: u64,
) -> Result<(), String> {
    if let Some(parent) = part.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| "MODEL_RUNTIME_DOWNLOAD_FAILED".to_string())?;
    }
    let existing = std::fs::metadata(part)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if existing == artifact.bytes {
        verify_sha256(part, &artifact.sha256, artifact.bytes)
            .map_err(|_| "MODEL_CHECKSUM_MISMATCH".to_string())?;
        return Ok(());
    }
    download_artifact_from_sources(
        app,
        client,
        model_id,
        &artifact.file_name,
        &artifact.urls,
        artifact.bytes,
        part,
        completed_bytes,
        total_bytes,
    )
    .await
    .map_err(|error| {
        if error == "MODEL_DOWNLOAD_FAILED" {
            "MODEL_RUNTIME_SOURCE_UNAVAILABLE".to_string()
        } else {
            error
        }
    })
}

async fn download_artifact_from_sources(
    app: Option<&AppHandle>,
    client: &reqwest::Client,
    model_id: &str,
    file_name: &str,
    urls: &[String],
    bytes: u64,
    part: &Path,
    completed_bytes: u64,
    total_bytes: u64,
) -> Result<(), String> {
    if urls.is_empty() {
        return Err("MODEL_RUNTIME_SOURCE_UNAVAILABLE".to_string());
    }
    for url in urls {
        match download_artifact_from_url(
            app,
            client,
            model_id,
            file_name,
            bytes,
            part,
            completed_bytes,
            total_bytes,
            url,
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(error) if error == "MODEL_DOWNLOAD_FAILED" => continue,
            Err(error) => return Err(error),
        }
    }
    Err("MODEL_DOWNLOAD_FAILED".to_string())
}

async fn download_artifact_from_url(
    app: Option<&AppHandle>,
    client: &reqwest::Client,
    model_id: &str,
    file_name: &str,
    expected_bytes: u64,
    part: &Path,
    completed_bytes: u64,
    total_bytes: u64,
    url: &str,
) -> Result<(), String> {
    let existing = std::fs::metadata(part)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let mut request = client.get(url);
    if existing > 0 && existing < expected_bytes {
        request = request.header(reqwest::header::RANGE, format!("bytes={existing}-"));
    }
    let response = request
        .send()
        .await
        .map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?;
    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
    {
        return Err("MODEL_DOWNLOAD_FAILED".to_string());
    }
    let append = existing > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    if append {
        let content_range = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| "MODEL_DOWNLOAD_FAILED".to_string())?;
        if !content_range.starts_with(&format!("bytes {existing}-")) {
            return Err("MODEL_DOWNLOAD_FAILED".to_string());
        }
    }
    let mut file = if append {
        tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(part)
            .await
            .map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?
    } else {
        tokio::fs::File::create(part)
            .await
            .map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?
    };
    let mut stream = response.bytes_stream();
    let mut downloaded = if append { existing } else { 0 };
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        wait_for_control(model_id).await?;
        let chunk = chunk.map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?;
        file.write_all(&chunk)
            .await
            .map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?;
        downloaded += chunk.len() as u64;
        if downloaded > expected_bytes {
            return Err("MODEL_CHECKSUM_MISMATCH".to_string());
        }
        emit_stage_optional(
            app,
            model_id,
            "downloading",
            file_name,
            completed_bytes.saturating_add(downloaded),
            total_bytes,
            "Downloading",
        );
    }
    file.flush()
        .await
        .map_err(|_| "MODEL_DOWNLOAD_FAILED".to_string())?;
    Ok(())
}

fn verify_sha256(path: &Path, expected: &str, expected_bytes: u64) -> Result<(), String> {
    let metadata = std::fs::metadata(path).map_err(|_| "MODEL_CORRUPTED".to_string())?;
    if metadata.len() != expected_bytes {
        return Err("MODEL_CHECKSUM_MISMATCH".to_string());
    }
    let mut file = std::fs::File::open(path).map_err(|_| "MODEL_CORRUPTED".to_string())?;
    let mut hasher = Sha256::new();
    // Keep the streaming buffer on the heap. Model verification can run from
    // the GUI entry thread, whose Windows stack is too small for 1 MiB locals.
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "MODEL_CORRUPTED".to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hex::encode(hasher.finalize());
    if digest != expected.to_ascii_lowercase() {
        return Err("MODEL_CHECKSUM_MISMATCH".to_string());
    }
    Ok(())
}

fn extract_zip(path: &Path, destination: &Path) -> Result<(), String> {
    let file =
        std::fs::File::open(path).map_err(|_| "MODEL_RUNTIME_ARCHIVE_INVALID".to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| "MODEL_RUNTIME_ARCHIVE_INVALID".to_string())?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| "MODEL_RUNTIME_ARCHIVE_INVALID".to_string())?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| "MODEL_RUNTIME_ARCHIVE_PATH_INVALID".to_string())?
            .to_path_buf();
        let output = destination.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&output)
                .map_err(|_| "MODEL_RUNTIME_INSTALL_FAILED".to_string())?;
            continue;
        }
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| "MODEL_RUNTIME_INSTALL_FAILED".to_string())?;
        }
        let mut target = std::fs::File::create(&output)
            .map_err(|_| "MODEL_RUNTIME_INSTALL_FAILED".to_string())?;
        std::io::copy(&mut entry, &mut target)
            .map_err(|_| "MODEL_RUNTIME_INSTALL_FAILED".to_string())?;
    }
    Ok(())
}

fn run_smoke_test(model_id: &str, model_dir: &Path, data_dir: &Path) -> Result<(), String> {
    let script = locate_sidecar_for_model(data_dir, model_id)
        .ok_or_else(|| "MODEL_RUNTIME_MISSING".to_string())?;
    if !crate::runtime::manifest_is_valid_for_startup(&script) {
        return Err("MODEL_RUNTIME_CORRUPTED".to_string());
    }
    // Resolve relative development paths before changing the child working
    // directory to the stable runtime directory.
    let script = std::fs::canonicalize(&script).unwrap_or(script);
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
        .arg(model_dir)
        .args([
            "--device",
            if model_id == "fun-asr-nano-2512" {
                "CPU"
            } else {
                "CUDA"
            },
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "MODEL_RUNTIME_FAILED".to_string())?;
    let silence = (0..16_000)
        .map(|index| (index as f32 * std::f32::consts::TAU * 220.0 / 16_000.0).sin() * 0.02)
        .collect::<Vec<_>>();
    let load = serde_json::json!({ "operation": "load", "requestId": "smoke-load", "modelId": model_id, "modelDir": model_dir, "device": if model_id == "fun-asr-nano-2512" { "CPU" } else { "CUDA" } });
    let transcribe = serde_json::json!({ "operation": "transcribe", "requestId": "smoke-transcribe", "modelId": model_id, "modelDir": model_dir, "device": if model_id == "fun-asr-nano-2512" { "CPU" } else { "CUDA" }, "language": "auto", "audioPcmF32LeBase64": STANDARD.encode(silence.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<_>>()) });
    for request in [
        load,
        transcribe,
        serde_json::json!({ "operation": "unload", "requestId": "smoke-unload" }),
    ] {
        writeln!(stdin, "{}", request).map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
    }
    drop(stdin);
    let output = child
        .wait_with_output()
        .map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
    if !output.status.success() {
        return Err("MODEL_SMOKE_TEST_FAILED".to_string());
    }
    let mut responses = 0;
    let mut transcribe_schema_ok = false;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|_| "MODEL_RUNTIME_FAILED".to_string())?;
        if value.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
            return Err(value
                .get("errorCode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("MODEL_SMOKE_TEST_FAILED")
                .to_string());
        }
        if value.get("requestId").and_then(serde_json::Value::as_str) == Some("smoke-transcribe") {
            let Some(items) = value.get("segments").and_then(serde_json::Value::as_array) else {
                return Err("MODEL_TIMESTAMP_TEST_FAILED".to_string());
            };
            let mut last_end = 0_i64;
            for item in items {
                let start = item
                    .get("startMs")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or_else(|| "MODEL_TIMESTAMP_TEST_FAILED".to_string())?;
                let end = item
                    .get("endMs")
                    .and_then(serde_json::Value::as_i64)
                    .ok_or_else(|| "MODEL_TIMESTAMP_TEST_FAILED".to_string())?;
                if start < 0 || end <= start || start < last_end {
                    return Err("MODEL_TIMESTAMP_TEST_FAILED".to_string());
                }
                last_end = end;
            }
            transcribe_schema_ok = true;
        }
        responses += 1;
    }
    if responses < 3 || !transcribe_schema_ok {
        return Err("MODEL_SMOKE_TEST_FAILED".to_string());
    }
    Ok(())
}

fn locate_sidecar_for_model(data_dir: &Path, model_id: &str) -> Option<PathBuf> {
    if model_id != "fun-asr-nano-2512" {
        if let Some(path) = locate_cuda_sidecar(data_dir) {
            return Some(path);
        }
    }
    locate_sidecar()
}

fn locate_cuda_sidecar(data_dir: &Path) -> Option<PathBuf> {
    if let Ok(value) = std::env::var("VERILECTURE_ASR_RUNTIME") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
    }
    let path = cuda_runtime_directory(data_dir).join(asr_runtime_executable_name());
    path.is_file().then_some(path)
}

fn probe_cuda_sidecar(script: &Path) -> bool {
    let script = std::fs::canonicalize(script).unwrap_or_else(|_| script.to_path_buf());
    let mut command = Command::new(script);
    crate::runtime::configure_child_command(&mut command);
    let Ok(output) = command.arg("--probe-cuda").output() else {
        return false;
    };
    output.status.success()
        && String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.trim() == "ASR_CUDA_USABLE=1")
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
    let candidates = python_executable_names()
        .iter()
        .flat_map(|name| {
            [
                executable_parent
                    .join("resources/asr-runtime/python")
                    .join(name),
                executable_parent.join("asr-runtime/python").join(name),
            ]
        })
        .collect::<Vec<_>>();
    if let Some(path) = candidates.into_iter().find(|path| path.is_file()) {
        return Some(path);
    }
    if cfg!(debug_assertions) {
        return Some(PathBuf::from("python"));
    }
    None
}

fn locate_sidecar() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("VERILECTURE_ASR_RUNTIME") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
    }
    let executable_parent = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let candidates = [
        PathBuf::from("src-tauri/resources/asr-runtime").join(asr_runtime_executable_name()),
        PathBuf::from("tools/asr/verilecture_asr_runtime.py"),
        PathBuf::from("../tools/asr/verilecture_asr_runtime.py"),
        executable_parent
            .join("resources/asr-runtime")
            .join(asr_runtime_executable_name()),
        executable_parent.join("resources/asr-runtime/verilecture_asr_runtime.py"),
        executable_parent.join("asr-runtime/verilecture_asr_runtime.py"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

fn is_sidecar_executable(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("exe"))
        .unwrap_or_else(|| cfg!(unix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read as TestRead, Write as TestWrite};
    use std::net::{Shutdown, TcpListener};
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    #[derive(Clone, Copy)]
    enum MockResponse {
        Normal,
        DropFirst(usize),
        Corrupt,
        NotFound,
    }

    struct MockHttpServer {
        url: String,
        ranges: Arc<Mutex<Vec<Option<u64>>>>,
        join: Option<JoinHandle<()>>,
    }

    impl Drop for MockHttpServer {
        fn drop(&mut self) {
            if let Some(join) = self.join.take() {
                join.join().expect("mock HTTP server thread");
            }
        }
    }

    fn start_mock_http_server(
        payload: Vec<u8>,
        response_mode: MockResponse,
        requests_to_serve: usize,
    ) -> MockHttpServer {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let ranges = Arc::new(Mutex::new(Vec::new()));
        let recorded_ranges = Arc::clone(&ranges);
        let join = thread::spawn(move || {
            for request_index in 0..requests_to_serve {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut request = Vec::new();
                let mut buffer = [0_u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    let Ok(read) = stream.read(&mut buffer) else {
                        break;
                    };
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if request.len() > 16 * 1024 {
                        break;
                    }
                }
                let request_text = String::from_utf8_lossy(&request);
                let range = request_text.lines().find_map(|line| {
                    line.strip_prefix("Range: bytes=")
                        .or_else(|| line.strip_prefix("range: bytes="))
                        .and_then(|value| value.split('-').next())
                        .and_then(|value| value.parse::<u64>().ok())
                });
                recorded_ranges.lock().unwrap().push(range);

                if matches!(response_mode, MockResponse::NotFound) {
                    let _ = stream.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    );
                    continue;
                }

                let start = range.unwrap_or(0) as usize;
                let mut body = payload.get(start..).unwrap_or_default().to_vec();
                if matches!(response_mode, MockResponse::Corrupt) && !body.is_empty() {
                    body[0] ^= 0xff;
                }
                let status = if range.is_some() {
                    "206 Partial Content"
                } else {
                    "200 OK"
                };
                let content_range = range
                    .map(|value| {
                        format!(
                            "Content-Range: bytes {}-{}/{}\r\n",
                            value,
                            value + body.len() as u64 - 1,
                            payload.len()
                        )
                    })
                    .unwrap_or_default();
                let header = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{content_range}Connection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let bytes_to_send = match response_mode {
                    MockResponse::DropFirst(limit) if request_index == 0 => limit.min(body.len()),
                    _ => body.len(),
                };
                let _ = stream.write_all(&body[..bytes_to_send]);
                let _ = stream.flush();
                if matches!(response_mode, MockResponse::DropFirst(_)) && request_index == 0 {
                    let _ = stream.shutdown(Shutdown::Both);
                }
            }
        });
        MockHttpServer {
            url: format!("http://{address}/runtime.zip"),
            ranges,
            join: Some(join),
        }
    }

    fn profile(vram_gib: u64, cuda: bool) -> HardwareProfile {
        HardwareProfile {
            os: "windows".to_string(),
            os_version: "Windows fixture".to_string(),
            architecture: "x86_64".to_string(),
            cpu_name: "fixture".to_string(),
            logical_cores: 8,
            avx2: true,
            total_ram_bytes: 16 * GB,
            available_ram_bytes: 8 * GB,
            disk_free_bytes: 20 * GB,
            nvidia_detected: cuda,
            gpu_name: cuda.then(|| "fixture GPU".to_string()),
            vram_bytes: cuda.then_some(vram_gib * GB),
            driver_version: None,
            nvidia_smi: cuda,
            cuda_driver_api: cuda,
            cuda_smoke_test: cuda,
            network_available: true,
            proxy_configured: false,
            proxy_source: None,
            model_directory_writable: true,
            scanned_at: "fixture".to_string(),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn routes_three_hardware_classes_without_fallback_hiding() {
        let high = model_options(Some(&profile(8, true)), Path::new("target/test-models"));
        assert!(
            high.iter()
                .find(|model| model.id == "qwen3-asr-1.7b")
                .unwrap()
                .supported
        );
        let medium = model_options(Some(&profile(6, true)), Path::new("target/test-models"));
        assert!(
            !medium
                .iter()
                .find(|model| model.id == "qwen3-asr-1.7b")
                .unwrap()
                .supported
        );
        assert!(
            medium
                .iter()
                .find(|model| model.id == "qwen3-asr-0.6b")
                .unwrap()
                .supported
        );
        let cpu = model_options(Some(&profile(0, false)), Path::new("target/test-models"));
        assert!(
            !cpu.iter()
                .find(|model| model.id == "qwen3-asr-0.6b")
                .unwrap()
                .supported
        );
        assert!(
            cpu.iter()
                .find(|model| model.id == "fun-asr-nano-2512")
                .unwrap()
                .supported
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn static_nvidia_profile_can_start_runtime_download_gate() {
        let mut profile = profile(12, true);
        profile.cuda_smoke_test = false;
        let options = model_options(Some(&profile), Path::new("target/test-models"));
        let high = options
            .iter()
            .find(|model| model.id == "qwen3-asr-1.7b")
            .unwrap();
        assert!(high.supported);
        assert!(high.recommended);
        assert!(high.reason.contains("runtime will be downloaded"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn unknown_vram_conservatively_disables_cuda_tiers() {
        let mut unknown = profile(8, true);
        unknown.vram_bytes = None;
        let options = model_options(Some(&unknown), Path::new("target/test-models"));
        assert!(
            !options
                .iter()
                .find(|model| model.id == "qwen3-asr-1.7b")
                .unwrap()
                .supported
        );
        assert!(
            !options
                .iter()
                .find(|model| model.id == "qwen3-asr-0.6b")
                .unwrap()
                .supported
        );
        assert!(
            options
                .iter()
                .find(|model| model.id == "fun-asr-nano-2512")
                .unwrap()
                .supported
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn non_windows_build_reports_missing_native_asr_runtime() {
        let options = model_options(Some(&profile(12, true)), Path::new("target/test-models"));
        assert!(options.iter().all(|model| !model.supported));
        assert!(options.iter().all(|model| !model.recommended));
        assert!(options
            .iter()
            .all(|model| model.reason.contains("本地 ASR 运行时")));
    }

    #[test]
    fn runtime_registry_rejects_pending_publication_sources() {
        let registry: RuntimeRegistry =
            serde_json::from_str(EMBEDDED_RUNTIME_REGISTRY).expect("embedded runtime registry");
        validate_runtime_registry(&registry).unwrap();
        let runtime = registry
            .runtimes
            .iter()
            .find(|runtime| runtime.id == CUDA_RUNTIME_ID)
            .unwrap();
        assert_eq!(runtime.status, "pending-publication");
        assert_eq!(
            published_runtime_artifact(runtime).unwrap_err(),
            "MODEL_RUNTIME_SOURCE_UNAVAILABLE"
        );
        assert_eq!(runtime.compressed_bytes, 4_617_121_514);
        assert_eq!(
            runtime.sha256,
            "4eafd198228821c9f5ca36ebd62a4ded53df6083ff1c3f8283127a8f9bc9a665"
        );
    }

    #[test]
    fn runtime_download_resumes_with_http_range_after_interruption() {
        let payload = (0..(128 * 1024))
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        let digest = hex::encode(Sha256::digest(&payload));
        let server = start_mock_http_server(payload.clone(), MockResponse::DropFirst(8192), 2);
        let model_id = format!("range-fixture-{}", uuid::Uuid::new_v4());
        let part =
            std::env::temp_dir().join(format!("verilecture-range-{}.part", uuid::Uuid::new_v4()));
        let client = reqwest::Client::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let first = runtime.block_on(download_artifact_from_sources(
            None,
            &client,
            &model_id,
            "fixture.zip",
            std::slice::from_ref(&server.url),
            payload.len() as u64,
            &part,
            0,
            payload.len() as u64,
        ));
        assert_eq!(first.unwrap_err(), "MODEL_DOWNLOAD_FAILED");
        let partial_bytes = std::fs::metadata(&part).unwrap().len();
        assert!(partial_bytes > 0 && partial_bytes < payload.len() as u64);
        runtime
            .block_on(download_artifact_from_sources(
                None,
                &client,
                &model_id,
                "fixture.zip",
                std::slice::from_ref(&server.url),
                payload.len() as u64,
                &part,
                0,
                payload.len() as u64,
            ))
            .unwrap();
        assert_eq!(std::fs::read(&part).unwrap(), payload);
        verify_sha256(&part, &digest, payload.len() as u64).unwrap();
        let ranges = server.ranges.lock().unwrap().clone();
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], None);
        assert_eq!(ranges[1], Some(partial_bytes));
        let _ = std::fs::remove_file(part);
        controls().lock().unwrap().remove(&model_id);
    }

    #[test]
    fn runtime_download_reports_size_and_sha_errors_before_install() {
        let payload = (0..8192)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        let digest = hex::encode(Sha256::digest(&payload));
        let client = reqwest::Client::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let corrupt_server = start_mock_http_server(payload.clone(), MockResponse::Corrupt, 1);
        let corrupt_part =
            std::env::temp_dir().join(format!("verilecture-corrupt-{}.part", uuid::Uuid::new_v4()));
        runtime
            .block_on(download_artifact_from_sources(
                None,
                &client,
                "corrupt-fixture",
                "fixture.zip",
                std::slice::from_ref(&corrupt_server.url),
                payload.len() as u64,
                &corrupt_part,
                0,
                payload.len() as u64,
            ))
            .unwrap();
        assert_eq!(
            verify_sha256(&corrupt_part, &digest, payload.len() as u64).unwrap_err(),
            "MODEL_CHECKSUM_MISMATCH"
        );
        let _ = std::fs::remove_file(corrupt_part);

        let size_server = start_mock_http_server(payload.clone(), MockResponse::Normal, 1);
        let size_part =
            std::env::temp_dir().join(format!("verilecture-size-{}.part", uuid::Uuid::new_v4()));
        runtime
            .block_on(download_artifact_from_sources(
                None,
                &client,
                "size-fixture",
                "fixture.zip",
                std::slice::from_ref(&size_server.url),
                payload.len() as u64 + 1,
                &size_part,
                0,
                payload.len() as u64 + 1,
            ))
            .unwrap();
        assert_eq!(
            verify_sha256(&size_part, &digest, payload.len() as u64 + 1).unwrap_err(),
            "MODEL_CHECKSUM_MISMATCH"
        );
        let _ = std::fs::remove_file(size_part);
    }

    #[test]
    fn runtime_http_404_is_not_treated_as_ready_source() {
        let payload = b"fixture".to_vec();
        let server = start_mock_http_server(payload.clone(), MockResponse::NotFound, 1);
        let artifact = RuntimeArtifact {
            file_name: "fixture.zip".to_string(),
            urls: vec![server.url.clone()],
            sha256: hex::encode(Sha256::digest(&payload)),
            bytes: payload.len() as u64,
        };
        let part =
            std::env::temp_dir().join(format!("verilecture-404-{}.part", uuid::Uuid::new_v4()));
        let client = reqwest::Client::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(download_runtime_artifact(
            None,
            &client,
            "runtime-404-fixture",
            &artifact,
            &part,
            0,
            payload.len() as u64,
        ));
        assert_eq!(result.unwrap_err(), "MODEL_RUNTIME_SOURCE_UNAVAILABLE");
        assert!(!part.exists());
    }

    #[test]
    fn partial_model_directory_is_not_ready() {
        let path =
            std::env::temp_dir().join(format!("verilecture-model-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(path.join("fun-asr-nano-2512")).unwrap();
        std::fs::write(path.join("fun-asr-nano-2512").join("READY.json"), br#"{"modelId":"fun-asr-nano-2512","registryVersion":"2026-07-31-qwen-fun-official","smokeTest":"passed","timestampTest":"passed"}"#).unwrap();
        assert!(!is_ready(&path, "fun-asr-nano-2512"));
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn sha256_verification_checks_size_and_content() {
        let path = std::env::temp_dir().join(format!("verilecture-sha-{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"verilecture").unwrap();
        let digest = hex::encode(Sha256::digest(b"verilecture"));
        assert!(verify_sha256(&path, &digest, 11).is_ok());
        assert_eq!(
            verify_sha256(&path, &digest, 12).unwrap_err(),
            "MODEL_CHECKSUM_MISMATCH"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn modelscope_mirror_precedes_hugging_face_and_github_has_no_mirror() {
        let artifact = &QWEN_06_ARTIFACTS[0];
        let urls = artifact_urls(artifact);
        assert_eq!(
            urls[0],
            "https://modelscope.cn/models/Qwen/Qwen3-ASR-0.6B/resolve/master/chat_template.json"
        );
        assert_eq!(urls[1], artifact.url);
        assert!(modelscope_mirror_url(
            "https://github.com/QwenAudio/Fun-ASR/releases/download/runtime-llamacpp-v0.1.9/funasr-llamacpp-windows-x64.zip"
        )
        .is_none());
    }

    #[test]
    fn resource_registry_covers_every_pinned_artifact() {
        let registry: serde_json::Value =
            serde_json::from_str(include_str!("../resources/model-registry.json")).unwrap();
        let models = registry
            .get("models")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        for model_id in ["qwen3-asr-1.7b", "qwen3-asr-0.6b", "fun-asr-nano-2512"] {
            let resource_model = models
                .iter()
                .find(|model| model.get("id").and_then(serde_json::Value::as_str) == Some(model_id))
                .unwrap();
            let resource_artifacts = resource_model
                .get("artifacts")
                .and_then(serde_json::Value::as_array)
                .unwrap();
            let pinned = artifacts_for(model_id);
            assert_eq!(resource_artifacts.len(), pinned.len(), "{model_id}");
            for artifact in pinned {
                let entry = resource_artifacts
                    .iter()
                    .find(|value| {
                        value.get("path").and_then(serde_json::Value::as_str)
                            == Some(artifact.file_name)
                    })
                    .unwrap_or_else(|| panic!("missing {} for {model_id}", artifact.file_name));
                assert_eq!(
                    entry.get("bytes").and_then(serde_json::Value::as_u64),
                    Some(artifact.bytes)
                );
                assert_eq!(
                    entry.get("sha256").and_then(serde_json::Value::as_str),
                    Some(artifact.sha256)
                );
            }
        }
    }

    #[test]
    fn cancelled_download_control_is_observed() {
        let model_id = format!("fixture-{}", uuid::Uuid::new_v4());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime
            .block_on(set_download_control(&model_id, "cancel"))
            .unwrap();
        let result = runtime.block_on(wait_for_control(&model_id));
        assert_eq!(result.unwrap_err(), "MODEL_DOWNLOAD_CANCELLED");
        controls().lock().unwrap().remove(&model_id);
    }
}
