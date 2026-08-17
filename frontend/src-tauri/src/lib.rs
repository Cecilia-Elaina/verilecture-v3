mod analysis;
mod audio;
mod db;
mod hardware;
mod lexicon;
mod models;
mod providers;
mod runtime;

use db::{init_database, AppDatabase};
use models::{model_options_with_states, selected_model_id, ModelOption};
use providers::{
    complete_json, delete_provider_secret, load_provider_secret, provider_from_row,
    save_provider_secret, test_provider_request,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

pub struct AppState {
    pub database: Mutex<AppDatabase>,
    pub data_dir: PathBuf,
    pub model_dir: PathBuf,
    pub hardware_scan: Mutex<()>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub provider: String,
    pub protocol: String,
    pub base_url: String,
    pub model_id: String,
    pub organization: Option<String>,
    pub timeout_seconds: u64,
    pub configured: bool,
    pub tested: bool,
    pub secret_ref: Option<String>,
    pub consent_granted: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub onboarding_complete: bool,
    pub locale: String,
    pub processing_mode: String,
    pub selected_model_id: Option<String>,
    pub hardware: Option<hardware::HardwareProfile>,
    pub models: Vec<ModelOption>,
    pub provider: Option<ProviderConfig>,
    pub model_directory: String,
    pub records: Vec<db::RecordSummary>,
    pub lexicons: Vec<db::LexiconSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub path: String,
    pub title: String,
    pub language: String,
    pub lexicon_id: Option<String>,
    #[serde(default)]
    pub job_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportLexiconRequest {
    pub path: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecordRequest {
    pub id: String,
    pub path: String,
    pub format: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LexiconUploadPreview {
    pub lexicon_id: String,
    pub selection: lexicon::UploadSelection,
}

#[tauri::command]
fn scan_hardware(state: State<'_, AppState>) -> Result<hardware::HardwareProfile, String> {
    scan_and_store_hardware(&state)
}

fn scan_and_store_hardware(state: &AppState) -> Result<hardware::HardwareProfile, String> {
    let _scan_guard = state
        .hardware_scan
        .lock()
        .map_err(|_| "HARDWARE_SCAN_BUSY".to_string())?;
    let profile = hardware::scan(&state.model_dir)?;
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .set_setting(
            "hardware_profile",
            &serde_json::to_string(&profile)
                .map_err(|_| "HARDWARE_PROFILE_INCOMPLETE".to_string())?,
        )?;
    Ok(profile)
}

#[tauri::command]
fn get_model_catalog(state: State<'_, AppState>) -> Result<Vec<ModelOption>, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    let profile = database
        .get_setting("hardware_profile")?
        .and_then(|value| serde_json::from_str::<hardware::HardwareProfile>(&value).ok());
    let states = models::model_states(&database)?;
    Ok(model_options_with_states(
        profile.as_ref(),
        &state.model_dir,
        &states,
    ))
}

#[tauri::command]
fn get_app_snapshot(state: State<'_, AppState>) -> Result<AppSnapshot, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    let profile = database
        .get_setting("hardware_profile")?
        .and_then(|value| serde_json::from_str::<hardware::HardwareProfile>(&value).ok());
    let selected = selected_model_id(&database)?;
    let provider = database.get_provider()?.map(provider_from_row);
    let onboarding_complete =
        database.get_setting("onboarding_complete")?.as_deref() == Some("true");
    let locale = database
        .get_setting("locale")?
        .unwrap_or_else(|| "zh-CN".to_string());
    Ok(AppSnapshot {
        onboarding_complete,
        locale,
        processing_mode: "local".to_string(),
        selected_model_id: selected,
        hardware: profile.clone(),
        models: model_options_with_states(
            profile.as_ref(),
            &state.model_dir,
            &models::model_states(&database)?,
        ),
        provider,
        model_directory: state.model_dir.to_string_lossy().to_string(),
        records: database.list_records()?,
        lexicons: database.list_lexicons()?,
    })
}

#[tauri::command]
async fn install_model(
    app: AppHandle,
    state: State<'_, AppState>,
    model_id: String,
) -> Result<(), String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .clone();
    let profile = database
        .get_setting("hardware_profile")?
        .and_then(|value| serde_json::from_str::<hardware::HardwareProfile>(&value).ok());
    if !models::is_supported(&model_id, profile.as_ref()) {
        return Err("MODEL_PROFILE_NOT_SUPPORTED".to_string());
    }
    models::install_model(
        &app,
        &database,
        &state.data_dir,
        &state.model_dir,
        &model_id,
    )
    .await?;
    let _refreshed_profile = scan_and_store_hardware(&state)?;
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .set_setting("selected_model_id", &model_id)
}

#[tauri::command]
fn select_model(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    let profile = database
        .get_setting("hardware_profile")?
        .and_then(|value| serde_json::from_str::<hardware::HardwareProfile>(&value).ok());
    if !models::is_supported(&model_id, profile.as_ref()) {
        return Err("MODEL_PROFILE_NOT_SUPPORTED".to_string());
    }
    if !models::is_ready(&state.model_dir, &model_id) {
        return Err("MODEL_NOT_INSTALLED".to_string());
    }
    database.set_setting("selected_model_id", &model_id)
}

#[tauri::command]
fn verify_model(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    match models::verify_model_integrity(&state.model_dir, &model_id) {
        Ok(()) => Ok(()),
        Err(error) => {
            let database = state
                .database
                .lock()
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
            database.record_model_install_event(
                &model_id,
                "corrupted",
                None,
                0,
                models::artifact_bytes_for_diagnostics(&model_id),
                Some(&error),
            )?;
            Err(error)
        }
    }
}

#[tauri::command]
fn complete_onboarding(state: State<'_, AppState>) -> Result<(), String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    let selected =
        selected_model_id(&database)?.ok_or_else(|| "MODEL_PROFILE_NOT_SELECTED".to_string())?;
    if !models::is_ready(&state.model_dir, &selected) {
        return Err("MODEL_NOT_INSTALLED".to_string());
    }
    let provider = database
        .get_provider()?
        .ok_or_else(|| "PROVIDER_NOT_CONFIGURED".to_string())?;
    if !provider.tested {
        return Err("PROVIDER_NOT_READY".to_string());
    }
    if !provider.consent_granted || !database.has_consent("cloud_llm_transcript")? {
        return Err("PROVIDER_CONSENT_REQUIRED".to_string());
    }
    if provider.secret_ref.is_none() {
        return Err("PROVIDER_SECRET_MISSING".to_string());
    }
    database.set_setting("onboarding_complete", "true")
}

#[tauri::command]
async fn pause_model_download(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    models::set_download_control(&model_id, "pause").await?;
    let state_snapshot = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .model_install_state(&model_id)?;
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .record_model_install_event(
            &model_id,
            "paused",
            state_snapshot
                .as_ref()
                .and_then(|value| value.file_name.as_deref()),
            state_snapshot
                .as_ref()
                .map(|value| value.downloaded_bytes)
                .unwrap_or_default(),
            state_snapshot
                .as_ref()
                .map(|value| value.total_bytes)
                .unwrap_or_default(),
            None,
        )
}

#[tauri::command]
async fn resume_model_download(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    models::set_download_control(&model_id, "resume").await?;
    let state_snapshot = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .model_install_state(&model_id)?;
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .record_model_install_event(
            &model_id,
            "downloading",
            state_snapshot
                .as_ref()
                .and_then(|value| value.file_name.as_deref()),
            state_snapshot
                .as_ref()
                .map(|value| value.downloaded_bytes)
                .unwrap_or_default(),
            state_snapshot
                .as_ref()
                .map(|value| value.total_bytes)
                .unwrap_or_default(),
            None,
        )
}

#[tauri::command]
async fn cancel_model_download(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    models::set_download_control(&model_id, "cancel").await?;
    let state_snapshot = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .model_install_state(&model_id)?;
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .record_model_install_event(
            &model_id,
            "cancelled",
            state_snapshot
                .as_ref()
                .and_then(|value| value.file_name.as_deref()),
            state_snapshot
                .as_ref()
                .map(|value| value.downloaded_bytes)
                .unwrap_or_default(),
            state_snapshot
                .as_ref()
                .map(|value| value.total_bytes)
                .unwrap_or_default(),
            None,
        )
}

#[tauri::command]
fn cancel_audio_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    audio::cancel_job(&job_id)?;
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .update_audio_job(
            &job_id,
            "cancelled",
            "cancelled",
            0,
            "JOB_CANCELLED",
            Some("JOB_CANCELLED"),
        )
}

#[tauri::command]
async fn test_text_provider(
    config: ProviderConfig,
    api_key: String,
) -> Result<providers::ProviderTestResult, String> {
    test_provider_request(&config, &api_key).await
}

#[tauri::command]
fn save_text_provider(
    state: State<'_, AppState>,
    config: ProviderConfig,
    api_key: String,
) -> Result<(), String> {
    if !config.tested {
        return Err("PROVIDER_NOT_READY".to_string());
    }
    if !config.consent_granted {
        return Err("PROVIDER_CONSENT_REQUIRED".to_string());
    }
    let old_secret = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .get_provider()?
        .and_then(|row| row.secret_ref);
    let secret_ref = format!("verilecture-v3/provider/{}", Uuid::new_v4());
    save_provider_secret(&secret_ref, &api_key)?;
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    database.save_provider(&config, &secret_ref)?;
    database.set_consent("cloud_llm_transcript", true)?;
    if let Some(old_secret) = old_secret {
        if old_secret != secret_ref {
            delete_provider_secret(&old_secret);
        }
    }
    Ok(())
}

#[tauri::command]
async fn import_audio(
    app: AppHandle,
    state: State<'_, AppState>,
    mut request: ImportRequest,
) -> Result<db::RecordDetail, String> {
    let job_id = request
        .job_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    request.job_id = Some(job_id.clone());
    let data_dir = state.data_dir.clone();
    let model_dir = state.model_dir.clone();
    let (model_id, provider, lexicon) = {
        let database = state
            .database
            .lock()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let model_id = selected_model_id(&database)?
            .ok_or_else(|| "MODEL_PROFILE_NOT_SELECTED".to_string())?;
        if !models::is_ready(&model_dir, &model_id) {
            return Err("MODEL_NOT_INSTALLED".to_string());
        }
        let provider = database.get_provider()?.map(|row| row.provider);
        let lexicon = request
            .lexicon_id
            .as_deref()
            .map(|id| database.get_lexicon_profile(id))
            .transpose()?
            .flatten();
        (model_id, provider, lexicon)
    };
    let source_filename = PathBuf::from(&request.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(&request.title)
        .to_string();
    audio::begin_job(&job_id)?;
    let start_result = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .start_audio_job(
            &job_id,
            &request.title,
            &source_filename,
            &request.language,
            &model_id,
            request.lexicon_id.as_deref(),
            lexicon.as_ref().map(|value| value.version),
        );
    if let Err(error) = start_result {
        audio::finish_job(&job_id);
        return Err(error);
    }
    let job_app = app.clone();
    let result = tokio::task::spawn_blocking(move || {
        audio::import_audio(
            &job_app,
            &data_dir,
            &model_dir,
            &request,
            &model_id,
            provider.as_deref(),
            lexicon.as_ref(),
        )
    })
    .await;
    let result = match result {
        Ok(value) => value,
        Err(_) => {
            audio::finish_job(&job_id);
            state
                .database
                .lock()
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
                .update_audio_job(
                    &job_id,
                    "failed",
                    "failed",
                    0,
                    "AUDIO_JOB_FAILED",
                    Some("AUDIO_JOB_FAILED"),
                )?;
            return Err("AUDIO_JOB_FAILED".to_string());
        }
    };
    let result = match result {
        Ok(value) => {
            audio::finish_job(&job_id);
            value
        }
        Err(error) => {
            audio::finish_job(&job_id);
            let status = if error == "JOB_CANCELLED" {
                "cancelled"
            } else {
                "failed"
            };
            let stage = if error == "JOB_CANCELLED" {
                "cancelled"
            } else {
                "failed"
            };
            state
                .database
                .lock()
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
                .update_audio_job(&job_id, status, stage, 0, &error, Some(&error))?;
            return Err(error);
        }
    };
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    database.insert_record(&result)?;
    database.update_audio_job(
        &job_id,
        "completed",
        "completed",
        100,
        "AUDIO_IMPORT_COMPLETED",
        None,
    )?;
    database
        .get_record(&result.id)?
        .ok_or_else(|| "DATABASE_OPERATION_FAILED".to_string())
}

#[tauri::command]
fn import_lexicon(
    state: State<'_, AppState>,
    request: ImportLexiconRequest,
) -> Result<db::LexiconSummary, String> {
    let parsed = lexicon::parse_and_copy(&state.data_dir, &request.path, request.name.as_deref())?;
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    database.insert_lexicon(
        &parsed.profile,
        &parsed.managed_path.to_string_lossy(),
        &parsed.file_type,
        parsed.extracted_chars,
        &parsed.extraction_quality,
        &parsed.chunks,
    )?;
    database
        .get_lexicon(&parsed.profile.id)?
        .ok_or_else(|| "DATABASE_OPERATION_FAILED".to_string())
}

#[tauri::command]
async fn generate_exam_points(
    state: State<'_, AppState>,
    record_id: String,
) -> Result<db::RecordDetail, String> {
    let (detail, provider, lexicon) = {
        let database = state
            .database
            .lock()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let detail = database
            .get_record(&record_id)?
            .ok_or_else(|| "RECORD_NOT_FOUND".to_string())?;
        if !database.has_consent("cloud_llm_transcript")? {
            return Err("PROVIDER_CONSENT_REQUIRED".to_string());
        }
        let provider = database
            .get_provider()?
            .map(provider_from_row)
            .ok_or_else(|| "PROVIDER_NOT_CONFIGURED".to_string())?;
        let lexicon = detail
            .lexicon_id
            .as_deref()
            .map(|id| database.get_lexicon_profile(id))
            .transpose()?
            .flatten();
        (detail, provider, lexicon)
    };
    if !provider.tested {
        return Err("PROVIDER_NOT_READY".to_string());
    }
    if !provider.consent_granted {
        return Err("PROVIDER_CONSENT_REQUIRED".to_string());
    }
    let secret_ref = provider
        .secret_ref
        .as_deref()
        .ok_or_else(|| "PROVIDER_SECRET_MISSING".to_string())?;
    let api_key = load_provider_secret(secret_ref)?;
    let outcome =
        analysis::generate_exam_points(&detail, lexicon.as_ref(), &provider, &api_key).await;
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    for run in &outcome.runs {
        database.insert_llm_run(run)?;
    }
    for audit in &outcome.audits {
        database.insert_payload_audit(audit)?;
    }
    if let Some(error) = outcome.error {
        return Err(error);
    }
    database.replace_exam_points(&record_id, &outcome.points)?;
    database
        .get_record(&record_id)?
        .ok_or_else(|| "RECORD_NOT_FOUND".to_string())
}

#[tauri::command]
fn get_lexicon_upload_preview(
    state: State<'_, AppState>,
    id: String,
) -> Result<LexiconUploadPreview, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    let profile = database
        .get_lexicon_profile(&id)?
        .ok_or_else(|| "LEXICON_NOT_FOUND".to_string())?;
    let total_chars = database.source_document_char_count(&profile.source_document_id)?;
    let chunks = database.source_chunks(&profile.source_document_id)?;
    let selection = lexicon::select_upload_chunks(&chunks, total_chars);
    Ok(LexiconUploadPreview {
        lexicon_id: id,
        selection,
    })
}

#[tauri::command]
fn set_privacy_consent(
    state: State<'_, AppState>,
    consent_type: String,
    granted: bool,
) -> Result<(), String> {
    if !matches!(
        consent_type.as_str(),
        "cloud_llm_transcript" | "cloud_llm_lexicon_structured_data" | "cloud_llm_textbook_excerpt"
    ) {
        return Err("PROVIDER_CONSENT_REQUIRED".to_string());
    }
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .set_consent(&consent_type, granted)
}

fn insert_lexicon_payload_audits(
    database: &db::AppDatabase,
    run_id: &str,
    source_document_id: &str,
    chunks: &[db::LexiconSourceChunk],
    total_document_chars: usize,
    provider_name: &str,
    output_chars: Option<i64>,
) -> Result<(), String> {
    for chunk in chunks {
        database.insert_payload_audit(&db::PayloadAudit {
            id: Uuid::new_v4().to_string(),
            llm_run_id: Some(run_id.to_string()),
            consent_type: "cloud_llm_textbook_excerpt".to_string(),
            document_id: Some(source_document_id.to_string()),
            chunk_id: Some(chunk.id.clone()),
            purpose: "lexicon_generation_excerpt".to_string(),
            sent_chars: chunk.char_count,
            total_document_chars: Some(total_document_chars as i64),
            provider_name: provider_name.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })?;
    }
    if let Some(output_chars) = output_chars {
        database.insert_payload_audit(&db::PayloadAudit {
            id: Uuid::new_v4().to_string(),
            llm_run_id: Some(run_id.to_string()),
            consent_type: "cloud_llm_lexicon_structured_data".to_string(),
            document_id: None,
            chunk_id: None,
            purpose: "lexicon_generation_structured_result".to_string(),
            sent_chars: output_chars,
            total_document_chars: Some(total_document_chars as i64),
            provider_name: provider_name.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })?;
    }
    Ok(())
}

#[tauri::command]
async fn generate_lexicon(
    state: State<'_, AppState>,
    id: String,
) -> Result<db::LexiconSummary, String> {
    let (profile, provider, chunks, total_document_chars) = {
        let database = state
            .database
            .lock()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        if !database.has_consent("cloud_llm_textbook_excerpt")?
            || !database.has_consent("cloud_llm_lexicon_structured_data")?
        {
            return Err("PROVIDER_CONSENT_REQUIRED".to_string());
        }
        let profile = database
            .get_lexicon_profile(&id)?
            .ok_or_else(|| "LEXICON_NOT_FOUND".to_string())?;
        let provider = database
            .get_provider()?
            .map(provider_from_row)
            .ok_or_else(|| "PROVIDER_NOT_CONFIGURED".to_string())?;
        if !provider.tested {
            return Err("PROVIDER_NOT_READY".to_string());
        }
        let total_document_chars =
            database.source_document_char_count(&profile.source_document_id)?;
        let all_chunks = database.source_chunks(&profile.source_document_id)?;
        let selection = lexicon::select_upload_chunks(&all_chunks, total_document_chars);
        if selection.chunks.is_empty() {
            return Err("TEXTBOOK_UPLOAD_LIMIT_EXCEEDED".to_string());
        }
        database.mark_source_chunks_selected(
            &profile.source_document_id,
            &selection
                .chunks
                .iter()
                .map(|chunk| chunk.id.clone())
                .collect::<Vec<_>>(),
        )?;
        (profile, provider, selection.chunks, total_document_chars)
    };
    let secret_ref = provider
        .secret_ref
        .as_deref()
        .ok_or_else(|| "PROVIDER_SECRET_MISSING".to_string())?;
    let api_key = load_provider_secret(secret_ref)?;
    let system = format!(
        "{}\n\n{}\n\nReturn exactly {{\"title\":string,\"chapters\":[],\"terms\":[]}}. Each term must include canonicalTerm, aliases, abbreviation, englishName, definition, chapterTitles, commonAsrErrors and sourceReferences. Do not return textbook prose.",
        include_str!("../resources/prompt-templates/textbook-metadata-v1.txt"),
        include_str!("../resources/prompt-templates/lexicon-terms-v1.txt")
    );
    let excerpt = chunks
        .iter()
        .map(|chunk| {
            format!(
                "[chunkId={} label={}]\n{}",
                chunk.id,
                chunk.source_label.as_deref().unwrap_or("local"),
                chunk.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let selected_chars = chunks
        .iter()
        .map(|chunk| chunk.char_count.max(0) as usize)
        .sum::<usize>();
    let user = format!(
        "Create a structured local lexicon from only the selected excerpt below. The full textbook stays local. Selected Unicode characters: {selected_chars}/{total_document_chars} (hard maximum 10%, capped at 120000).\n\n{excerpt}"
    );
    let run_id = Uuid::new_v4().to_string();
    let started = chrono::Utc::now();
    let input_chars = system.chars().count() + user.chars().count();
    let record_failed =
        |error_code: &str, output_chars: i64, duration_ms: i64| -> Result<(), String> {
            let database = state
                .database
                .lock()
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
            database.insert_llm_run(&db::LlmRunAudit {
                id: run_id.clone(),
                purpose: "lexicon_generation".to_string(),
                provider_name: provider.provider.clone(),
                model_id: provider.model_id.clone(),
                status: "failed".to_string(),
                input_chars: input_chars as i64,
                output_chars,
                duration_ms,
                error_code: Some(error_code.to_string()),
                created_at: started.to_rfc3339(),
            })?;
            insert_lexicon_payload_audits(
                &database,
                &run_id,
                &profile.source_document_id,
                &chunks,
                total_document_chars,
                &provider.provider,
                (output_chars > 0).then_some(output_chars),
            )
        };
    let completion = complete_json(&provider, &api_key, &system, &user, 8_192).await;
    let (generated, output_chars, duration_ms) = match completion {
        Ok(response) => {
            let output_chars = response.text.chars().count() as i64;
            let parsed = match serde_json::from_str::<lexicon::GeneratedLexicon>(&response.text) {
                Ok(parsed) => parsed,
                Err(_) => {
                    let error = "LEXICON_GENERATION_FAILED".to_string();
                    record_failed(&error, output_chars, response.duration_ms)?;
                    return Err(error);
                }
            };
            (parsed, output_chars, response.duration_ms)
        }
        Err(error) => {
            record_failed(
                &error,
                0,
                chrono::Utc::now()
                    .signed_duration_since(started)
                    .num_milliseconds()
                    .max(0),
            )?;
            return Err(error);
        }
    };
    let mut next_profile = match lexicon::merge_generated_profile(&profile, generated) {
        Ok(profile) => profile,
        Err(_) => {
            let error = "LEXICON_GENERATION_FAILED".to_string();
            record_failed(&error, output_chars, duration_ms)?;
            return Err(error);
        }
    };
    next_profile.updated_at = chrono::Utc::now().to_rfc3339();
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    database.insert_llm_run(&db::LlmRunAudit {
        id: run_id.clone(),
        purpose: "lexicon_generation".to_string(),
        provider_name: provider.provider.clone(),
        model_id: provider.model_id.clone(),
        status: "completed".to_string(),
        input_chars: input_chars as i64,
        output_chars,
        duration_ms,
        error_code: None,
        created_at: started.to_rfc3339(),
    })?;
    insert_lexicon_payload_audits(
        &database,
        &run_id,
        &profile.source_document_id,
        &chunks,
        total_document_chars,
        &provider.provider,
        Some(output_chars),
    )?;
    database.update_lexicon_profile(&next_profile)?;
    database
        .get_lexicon(&id)?
        .ok_or_else(|| "LEXICON_NOT_FOUND".to_string())
}

#[tauri::command]
fn export_record(state: State<'_, AppState>, request: ExportRecordRequest) -> Result<(), String> {
    let detail = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .get_record(&request.id)?
        .ok_or_else(|| "RECORD_NOT_FOUND".to_string())?;
    let format = request.format.to_ascii_lowercase();
    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&detail).map_err(|_| "EXPORT_FAILED".to_string())?,
        "md" | "markdown" => export_markdown(&detail),
        "txt" => export_text(&detail),
        _ => return Err("EXPORT_FORMAT_UNSUPPORTED".to_string()),
    };
    let path = PathBuf::from(&request.path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "EXPORT_FAILED".to_string())?;
    }
    let temporary = path.with_extension(format!(
        "{}.part",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("export")
    ));
    std::fs::write(&temporary, content.as_bytes()).map_err(|_| "EXPORT_FAILED".to_string())?;
    std::fs::rename(&temporary, &path).map_err(|_| {
        let _ = std::fs::remove_file(&temporary);
        "EXPORT_FAILED".to_string()
    })
}

fn export_markdown(detail: &db::RecordDetail) -> String {
    let mut output = format!(
        "# {}\n\n- Duration: {} ms\n- Model: {}\n- Language: {}\n\n## Exam points\n\n",
        detail.title, detail.duration_ms, detail.model_id, detail.language
    );
    if detail.exam_points.is_empty() {
        output.push_str("No exam points generated.\n");
    } else {
        for point in &detail.exam_points {
            output.push_str(&format!(
                "### {}\n\n{}–{} ms\n\n{}\n\nEvidence: `{}`\n\n",
                point.title,
                point.start_ms,
                point.end_ms,
                point.detail,
                point.segment_ids.join("`, `")
            ));
        }
    }
    output.push_str("## Calibrated transcript\n\n");
    for segment in &detail.calibrated_segments {
        output.push_str(&format!(
            "- **{}–{} ms** {}\n",
            segment.start_ms, segment.end_ms, segment.text
        ));
    }
    output.push_str("\n## Raw transcript\n\n");
    for segment in &detail.raw_segments {
        output.push_str(&format!(
            "- **{}–{} ms** {}\n",
            segment.start_ms, segment.end_ms, segment.text
        ));
    }
    output
}

fn export_text(detail: &db::RecordDetail) -> String {
    detail
        .calibrated_segments
        .iter()
        .map(|segment| format!("[{}–{}] {}", segment.start_ms, segment.end_ms, segment.text))
        .collect::<Vec<_>>()
        .join("\n")
}

#[tauri::command]
fn delete_lexicon(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .soft_delete_lexicon(&id)
}

#[tauri::command]
fn get_lexicon(state: State<'_, AppState>, id: String) -> Result<db::LexiconProfile, String> {
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .get_lexicon_profile(&id)?
        .ok_or_else(|| "LEXICON_NOT_FOUND".to_string())
}

#[tauri::command]
fn update_lexicon(
    state: State<'_, AppState>,
    mut profile: db::LexiconProfile,
) -> Result<db::LexiconSummary, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    let current = database
        .get_lexicon_profile(&profile.id)?
        .ok_or_else(|| "LEXICON_NOT_FOUND".to_string())?;
    if profile.source_document_id != current.source_document_id {
        return Err("LEXICON_VERSION_NOT_FOUND".to_string());
    }
    profile.version = current.version.saturating_add(1);
    profile.created_at = current.created_at;
    profile.updated_at = chrono::Utc::now().to_rfc3339();
    profile.name = profile.name.trim().chars().take(300).collect();
    profile.textbook_title = profile.textbook_title.trim().chars().take(300).collect();
    if profile.name.is_empty() || profile.textbook_title.is_empty() {
        return Err("LEXICON_VERSION_NOT_FOUND".to_string());
    }
    profile
        .terms
        .retain(|term| !term.canonical_term.trim().is_empty());
    for term in &mut profile.terms {
        term.canonical_term = term.canonical_term.trim().chars().take(200).collect();
        term.confirmed_by_user = true;
    }
    database.update_lexicon_profile(&profile)?;
    database
        .get_lexicon(&profile.id)?
        .ok_or_else(|| "LEXICON_NOT_FOUND".to_string())
}

#[tauri::command]
fn list_records(state: State<'_, AppState>) -> Result<Vec<db::RecordSummary>, String> {
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .list_records()
}

#[tauri::command]
fn get_record(state: State<'_, AppState>, id: String) -> Result<db::RecordDetail, String> {
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .get_record(&id)?
        .ok_or_else(|| "DATABASE_OPERATION_FAILED".to_string())
}

#[tauri::command]
fn delete_record(state: State<'_, AppState>, id: String, delete_copy: bool) -> Result<(), String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    if delete_copy {
        if let Some(record) = database.get_record(&id)? {
            if let Some(path) = record.audio_path {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    database.delete_record(&id)
}

#[tauri::command]
fn list_lexicons(state: State<'_, AppState>) -> Result<Vec<db::LexiconSummary>, String> {
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .list_lexicons()
}

#[tauri::command]
fn set_locale(state: State<'_, AppState>, locale: String) -> Result<(), String> {
    if locale != "zh-CN" && locale != "en-US" {
        return Err("LOCALE_UNSUPPORTED".to_string());
    }
    state
        .database
        .lock()
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
        .set_setting("locale", &locale)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            let model_dir = data_dir.join("models");
            std::fs::create_dir_all(&model_dir).map_err(|error| error.to_string())?;
            let database = init_database(&data_dir.join("verilecture_v3.sqlite"))
                .map_err(|error| error.to_string())?;
            app.manage(AppState {
                database: Mutex::new(database),
                data_dir,
                model_dir,
                hardware_scan: Mutex::new(()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_hardware,
            get_model_catalog,
            get_app_snapshot,
            install_model,
            select_model,
            verify_model,
            complete_onboarding,
            pause_model_download,
            resume_model_download,
            cancel_model_download,
            cancel_audio_job,
            test_text_provider,
            save_text_provider,
            import_audio,
            import_lexicon,
            generate_exam_points,
            get_lexicon_upload_preview,
            set_privacy_consent,
            generate_lexicon,
            export_record,
            delete_lexicon,
            get_lexicon,
            update_lexicon,
            list_records,
            get_record,
            delete_record,
            list_lexicons,
            set_locale
        ])
        .run(tauri::generate_context!())
        .expect("error while running VeriLecture");
}
