use crate::ProviderConfig;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppDatabase {
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProviderRow {
    pub provider: String,
    pub protocol: String,
    pub base_url: String,
    pub model_id: String,
    pub organization: Option<String>,
    pub timeout_seconds: u64,
    pub tested: bool,
    pub secret_ref: Option<String>,
    pub consent_granted: bool,
}

#[derive(Debug, Clone)]
pub struct ModelInstallState {
    pub stage: String,
    pub file_name: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub error_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
    pub language: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExamPoint {
    pub id: String,
    pub chapter_id: Option<String>,
    pub chapter_title: String,
    pub title: String,
    pub detail: String,
    #[serde(skip_serializing, default)]
    pub kind: String,
    pub segment_ids: Vec<String>,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub duration_ms: i64,
    pub status: String,
    pub model_id: String,
    pub provider_name: Option<String>,
    pub lexicon_name: Option<String>,
    pub source_path: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordDetail {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub duration_ms: i64,
    pub status: String,
    pub model_id: String,
    pub provider_name: Option<String>,
    pub lexicon_name: Option<String>,
    pub lexicon_id: Option<String>,
    pub source_path: Option<String>,
    pub audio_path: Option<String>,
    pub language: String,
    pub raw_segments: Vec<TranscriptSegment>,
    pub calibrated_segments: Vec<TranscriptSegment>,
    pub exam_points: Vec<ExamPoint>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexiconSummary {
    pub id: String,
    pub name: String,
    pub textbook_title: String,
    pub version: i64,
    pub terminology_count: i64,
    pub chapter_count: i64,
    pub updated_at: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub order: i64,
    pub title: String,
    pub label: Option<String>,
    pub source_document_id: String,
    pub source_page: Option<i64>,
    pub source_slide: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexiconTerm {
    pub id: String,
    pub canonical_term: String,
    pub aliases: Vec<String>,
    pub abbreviation: Option<String>,
    pub english_name: Option<String>,
    pub definition: Option<String>,
    pub chapter_ids: Vec<String>,
    pub common_asr_errors: Vec<String>,
    pub source_references: Vec<String>,
    pub confirmed_by_user: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CorrectionRule {
    pub id: String,
    pub original_text: String,
    pub corrected_text: String,
    pub enabled: bool,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexiconProfile {
    pub id: String,
    pub name: String,
    pub version: i64,
    pub textbook_title: String,
    pub source_document_id: String,
    pub chapters: Vec<ChapterNode>,
    pub terms: Vec<LexiconTerm>,
    pub correction_rules: Vec<CorrectionRule>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LexiconSourceChunk {
    pub id: String,
    pub ordinal: i64,
    pub source_label: Option<String>,
    pub text: String,
    pub char_count: i64,
    pub selected_for_upload: bool,
}

#[derive(Debug, Clone)]
pub struct LlmRunAudit {
    pub id: String,
    pub purpose: String,
    pub provider_name: String,
    pub model_id: String,
    pub status: String,
    pub input_chars: i64,
    pub output_chars: i64,
    pub duration_ms: i64,
    pub error_code: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct PayloadAudit {
    pub id: String,
    pub llm_run_id: Option<String>,
    pub consent_type: String,
    pub document_id: Option<String>,
    pub chunk_id: Option<String>,
    pub purpose: String,
    pub sent_chars: i64,
    pub total_document_chars: Option<i64>,
    pub provider_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct ImportedRecord {
    pub id: String,
    pub title: String,
    pub source_path: Option<String>,
    pub audio_path: Option<String>,
    pub created_at: String,
    pub duration_ms: i64,
    pub status: String,
    pub model_id: String,
    pub provider_name: Option<String>,
    pub lexicon_id: Option<String>,
    pub lexicon_version: Option<i64>,
    pub language: String,
    pub raw_segments: Vec<TranscriptSegment>,
    pub calibrated_segments: Vec<TranscriptSegment>,
    pub exam_points: Vec<ExamPoint>,
}

pub fn init_database(path: &Path) -> Result<AppDatabase, rusqlite::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| rusqlite::Error::InvalidPath(parent.to_path_buf()))?;
    }
    let connection = Connection::open(path)?;
    connection.execute_batch(include_str!("../migrations/001_init.sql"))?;
    ensure_column(
        &connection,
        "provider_configs",
        "consent_granted",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        &connection,
        "exam_points",
        "kind",
        "TEXT NOT NULL DEFAULT 'inferred'",
    )?;
    Ok(AppDatabase {
        path: path.to_path_buf(),
    })
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), rusqlite::Error> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    let exists = columns.flatten().any(|value| value == column);
    if !exists {
        connection.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
            [],
        )?;
    }
    Ok(())
}

impl AppDatabase {
    fn connection(&self) -> Result<Connection, String> {
        Connection::open(&self.path).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let connection = self.connection()?;
        connection.execute("INSERT INTO app_settings(key,value,updated_at) VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at", params![key, value, Utc::now().to_rfc3339()]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT value FROM app_settings WHERE key=?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn set_consent(&self, consent_type: &str, granted: bool) -> Result<(), String> {
        let connection = self.connection()?;
        let now = Utc::now().to_rfc3339();
        connection.execute("INSERT INTO privacy_consents(consent_type,granted,granted_at,revoked_at,updated_at) VALUES(?1,?2,CASE WHEN ?2=1 THEN ?3 ELSE NULL END,CASE WHEN ?2=0 THEN ?3 ELSE NULL END,?3) ON CONFLICT(consent_type) DO UPDATE SET granted=excluded.granted,granted_at=excluded.granted_at,revoked_at=excluded.revoked_at,updated_at=excluded.updated_at", params![consent_type, granted as i64, now]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn has_consent(&self, consent_type: &str) -> Result<bool, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT granted FROM privacy_consents WHERE consent_type=?1",
                params![consent_type],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map(|value| value.unwrap_or(0) != 0)
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn save_provider(&self, config: &ProviderConfig, secret_ref: &str) -> Result<(), String> {
        let connection = self.connection()?;
        let now = Utc::now().to_rfc3339();
        connection
            .execute("DELETE FROM provider_configs", [])
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        connection.execute("INSERT INTO provider_configs(id,provider,protocol,base_url,model_id,organization,timeout_seconds,tested,secret_ref,consent_granted,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?11)", params![uuid::Uuid::new_v4().to_string(), config.provider, config.protocol, config.base_url, config.model_id, config.organization, config.timeout_seconds as i64, config.tested as i64, secret_ref, config.consent_granted as i64, now]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn get_provider(&self) -> Result<Option<ProviderRow>, String> {
        let connection = self.connection()?;
        connection.query_row("SELECT provider,protocol,base_url,model_id,organization,timeout_seconds,tested,secret_ref,consent_granted FROM provider_configs ORDER BY updated_at DESC LIMIT 1", [], |row| Ok(ProviderRow { provider: row.get(0)?, protocol: row.get(1)?, base_url: row.get(2)?, model_id: row.get(3)?, organization: row.get(4)?, timeout_seconds: row.get::<_, i64>(5)?.max(1) as u64, tested: row.get::<_, i64>(6)? != 0, secret_ref: row.get(7)?, consent_granted: row.get::<_, i64>(8)? != 0 })).optional().map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn record_model_install_event(
        &self,
        model_id: &str,
        stage: &str,
        file_name: Option<&str>,
        downloaded_bytes: u64,
        total_bytes: u64,
        error_code: Option<&str>,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO model_install_events(id,model_id,stage,file_name,downloaded_bytes,total_bytes,error_code,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    Uuid::new_v4().to_string(),
                    model_id,
                    stage,
                    file_name,
                    downloaded_bytes as i64,
                    total_bytes as i64,
                    error_code,
                    Utc::now().to_rfc3339()
                ],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn model_install_state(&self, model_id: &str) -> Result<Option<ModelInstallState>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT stage,file_name,downloaded_bytes,total_bytes,error_code FROM model_install_events WHERE model_id=?1 ORDER BY created_at DESC LIMIT 1",
                params![model_id],
                |row| {
                    Ok(ModelInstallState {
                        stage: row.get(0)?,
                        file_name: row.get(1)?,
                        downloaded_bytes: row.get::<_, i64>(2)?.max(0) as u64,
                        total_bytes: row.get::<_, i64>(3)?.max(0) as u64,
                        error_code: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn save_installed_model(
        &self,
        model_id: &str,
        registry_version: &str,
        install_path: &str,
        manifest_json: &str,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "DELETE FROM installed_model_profiles WHERE model_id=?1",
                params![model_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        connection
            .execute(
                "INSERT INTO installed_model_profiles(id,model_id,registry_version,install_path,status,manifest_json,last_verified_at,created_at,updated_at) VALUES(?1,?2,?3,?4,'ready',?5,?6,?6,?6)",
                params![
                    Uuid::new_v4().to_string(),
                    model_id,
                    registry_version,
                    install_path,
                    manifest_json,
                    now
                ],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn start_audio_job(
        &self,
        job_id: &str,
        title: &str,
        source_filename: &str,
        language: &str,
        model_id: &str,
        lexicon_id: Option<&str>,
        lexicon_version: Option<i64>,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "INSERT OR REPLACE INTO audio_jobs(id,title,status,source_filename,language_preference,asr_profile_id,lexicon_profile_id,lexicon_version,current_stage,progress_percent,created_at,updated_at) VALUES(?1,?2,'processing',?3,?4,?5,?6,?7,'queued',0,?8,?8)",
                params![job_id, title, source_filename, language, model_id, lexicon_id, lexicon_version, now],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        connection
            .execute(
                "INSERT INTO job_events(id,job_id,stage,progress_percent,message_code,created_at) VALUES(?1,?2,'queued',0,'AUDIO_JOB_QUEUED',?3)",
                params![Uuid::new_v4().to_string(), job_id, now],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn update_audio_job(
        &self,
        job_id: &str,
        status: &str,
        stage: &str,
        progress_percent: i64,
        message_code: &str,
        error_code: Option<&str>,
    ) -> Result<(), String> {
        let connection = self.connection()?;
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "UPDATE audio_jobs SET status=?1,current_stage=?2,progress_percent=?3,error_code=?4,error_message_safe=?5,updated_at=?6,completed_at=CASE WHEN ?1 IN ('completed','failed','cancelled') THEN ?6 ELSE completed_at END WHERE id=?7",
                params![status, stage, progress_percent.clamp(0, 100), error_code, message_code, now, job_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        connection
            .execute(
                "INSERT INTO job_events(id,job_id,stage,progress_percent,message_code,error_code,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    Uuid::new_v4().to_string(),
                    job_id,
                    stage,
                    progress_percent.clamp(0, 100),
                    message_code,
                    error_code,
                    now
                ],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn insert_record(&self, record: &ImportedRecord) -> Result<(), String> {
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let source_filename = record
            .source_path
            .as_deref()
            .and_then(|path| Path::new(path).file_name())
            .and_then(|value| value.to_str())
            .unwrap_or(&record.title);
        let file_size = record
            .audio_path
            .as_deref()
            .and_then(|path| std::fs::metadata(path).ok())
            .map(|metadata| metadata.len() as i64)
            .unwrap_or_default();
        transaction.execute("INSERT OR REPLACE INTO audio_jobs(id,title,status,source_filename,managed_audio_path,duration_ms,file_size_bytes,audio_format,language_preference,asr_profile_id,lexicon_profile_id,lexicon_version,current_stage,progress_percent,created_at,updated_at,completed_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?15,?15)", params![record.id, record.title, record.status, source_filename, record.audio_path, record.duration_ms, file_size, record.audio_path.as_deref().and_then(|path| Path::new(path).extension()).and_then(|value| value.to_str()), record.language, record.model_id, record.lexicon_id, record.lexicon_version, "completed", 100, record.created_at]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        if let Some(audio_path) = &record.audio_path {
            transaction.execute("INSERT INTO audio_assets(id,job_id,source_path_display,managed_path,is_original_copy,created_at) VALUES(?1,?2,?3,?4,?5,?6)", params![Uuid::new_v4().to_string(), record.id, record.source_path, audio_path, 1, record.created_at]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        transaction.execute("INSERT INTO job_events(id,job_id,stage,progress_percent,message_code,created_at) VALUES(?1,?2,?3,?4,?5,?6)", params![Uuid::new_v4().to_string(), record.id, "completed", 100, "AUDIO_IMPORT_COMPLETED", record.created_at]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let raw_version_id = Uuid::new_v4().to_string();
        let calibrated_version_id = Uuid::new_v4().to_string();
        transaction.execute("INSERT INTO transcript_versions(id,job_id,version_type,parent_version_id,lexicon_id,lexicon_version,created_at) VALUES(?1,?2,?3,NULL,?4,?5,?6)", params![raw_version_id, record.id, "raw", record.lexicon_id, record.lexicon_version, record.created_at]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction.execute("INSERT INTO transcript_versions(id,job_id,version_type,parent_version_id,lexicon_id,lexicon_version,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![calibrated_version_id, record.id, "calibrated", raw_version_id, record.lexicon_id, record.lexicon_version, record.created_at]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction.execute("INSERT INTO audio_records(id,title,source_path,audio_path,created_at,duration_ms,status,model_id,provider_name,lexicon_id,lexicon_version,language) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)", params![record.id, record.title, record.source_path, record.audio_path, record.created_at, record.duration_ms, record.status, record.model_id, record.provider_name, record.lexicon_id, record.lexicon_version, record.language]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        for segment in record
            .raw_segments
            .iter()
            .chain(record.calibrated_segments.iter())
        {
            transaction.execute("INSERT INTO transcript_segments(id,record_id,start_ms,end_ms,text,language,source) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![segment.id, record.id, segment.start_ms, segment.end_ms, segment.text, segment.language, segment.source]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        for point in &record.exam_points {
            transaction.execute("INSERT INTO exam_points(id,record_id,chapter_id,chapter_title,title,detail,kind,segment_ids_json,start_ms,end_ms) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![point.id, record.id, point.chapter_id, point.chapter_title, point.title, point.detail, point.kind, serde_json::to_string(&point.segment_ids).map_err(|_| "DATABASE_OPERATION_FAILED")?, point.start_ms, point.end_ms]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        write_normalized_exam_points(
            &transaction,
            &record.id,
            Some(&calibrated_version_id),
            record.lexicon_id.as_deref(),
            record.lexicon_version,
            &record.exam_points,
            &record.created_at,
        )?;
        transaction
            .commit()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn list_records(&self) -> Result<Vec<RecordSummary>, String> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT r.id,r.title,r.created_at,r.duration_ms,r.status,r.model_id,r.provider_name,l.name,r.source_path FROM audio_records r LEFT JOIN lexicons l ON l.id=r.lexicon_id ORDER BY r.created_at DESC").map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(RecordSummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    duration_ms: row.get(3)?,
                    status: row.get(4)?,
                    model_id: row.get(5)?,
                    provider_name: row.get(6)?,
                    lexicon_name: row.get(7)?,
                    source_path: row.get(8)?,
                })
            })
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn get_record(&self, id: &str) -> Result<Option<RecordDetail>, String> {
        let connection = self.connection()?;
        let base = connection.query_row("SELECT r.id,r.title,r.created_at,r.duration_ms,r.status,r.model_id,r.provider_name,l.name,r.lexicon_id,r.source_path,r.audio_path,r.language FROM audio_records r LEFT JOIN lexicons l ON l.id=r.lexicon_id WHERE r.id=?1", params![id], |row| Ok(RecordDetail { id: row.get(0)?, title: row.get(1)?, created_at: row.get(2)?, duration_ms: row.get(3)?, status: row.get(4)?, model_id: row.get(5)?, provider_name: row.get(6)?, lexicon_name: row.get(7)?, lexicon_id: row.get(8)?, source_path: row.get(9)?, audio_path: row.get(10)?, language: row.get(11)?, raw_segments: Vec::new(), calibrated_segments: Vec::new(), exam_points: Vec::new() })).optional().map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let Some(mut detail) = base else {
            return Ok(None);
        };
        let mut segments = connection.prepare("SELECT id,start_ms,end_ms,text,language,source FROM transcript_segments WHERE record_id=?1 ORDER BY start_ms,id").map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let segment_rows = segments
            .query_map(params![id], |row| {
                Ok(TranscriptSegment {
                    id: row.get(0)?,
                    start_ms: row.get(1)?,
                    end_ms: row.get(2)?,
                    text: row.get(3)?,
                    language: row.get(4)?,
                    source: row.get(5)?,
                })
            })
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        for segment in segment_rows.flatten() {
            if segment.source == "raw" {
                detail.raw_segments.push(segment);
            } else {
                detail.calibrated_segments.push(segment);
            }
        }
        let mut points = connection.prepare("SELECT id,chapter_id,chapter_title,title,detail,kind,segment_ids_json,start_ms,end_ms FROM exam_points WHERE record_id=?1 ORDER BY start_ms,id").map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let point_rows = points
            .query_map(params![id], |row| {
                let ids: String = row.get(6)?;
                Ok(ExamPoint {
                    id: row.get(0)?,
                    chapter_id: row.get(1)?,
                    chapter_title: row.get(2)?,
                    title: row.get(3)?,
                    detail: row.get(4)?,
                    kind: row.get(5)?,
                    segment_ids: serde_json::from_str(&ids).unwrap_or_default(),
                    start_ms: row.get(7)?,
                    end_ms: row.get(8)?,
                })
            })
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        detail.exam_points = point_rows.flatten().collect();
        Ok(Some(detail))
    }

    pub fn replace_exam_points(&self, record_id: &str, points: &[ExamPoint]) -> Result<(), String> {
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let normalized = transaction
            .query_row(
                "SELECT lexicon_profile_id,lexicon_version FROM audio_jobs WHERE id=?1",
                params![record_id],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "DELETE FROM exam_point_segments WHERE point_id IN (SELECT id FROM exam_points WHERE record_id=?1)",
                params![record_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "DELETE FROM exam_point_audio_ranges WHERE point_id IN (SELECT id FROM exam_points WHERE record_id=?1)",
                params![record_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "DELETE FROM exam_point_source_refs WHERE point_id IN (SELECT id FROM exam_points WHERE record_id=?1)",
                params![record_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "DELETE FROM exam_point_sets WHERE job_id=?1",
                params![record_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "DELETE FROM exam_points WHERE record_id=?1",
                params![record_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        for point in points {
            transaction
                .execute("INSERT INTO exam_points(id,record_id,chapter_id,chapter_title,title,detail,kind,segment_ids_json,start_ms,end_ms) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![point.id, record_id, point.chapter_id, point.chapter_title, point.title, point.detail, point.kind, serde_json::to_string(&point.segment_ids).map_err(|_| "DATABASE_OPERATION_FAILED")?, point.start_ms, point.end_ms])
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        if let Some((lexicon_id, lexicon_version)) = normalized {
            let calibrated_version_id = transaction
                .query_row(
                    "SELECT id FROM transcript_versions WHERE job_id=?1 AND version_type='calibrated' ORDER BY created_at DESC LIMIT 1",
                    params![record_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
            write_normalized_exam_points(
                &transaction,
                record_id,
                calibrated_version_id.as_deref(),
                lexicon_id.as_deref(),
                lexicon_version,
                points,
                &Utc::now().to_rfc3339(),
            )?;
        }
        transaction
            .commit()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn insert_llm_run(&self, run: &LlmRunAudit) -> Result<(), String> {
        let connection = self.connection()?;
        connection.execute("INSERT INTO llm_runs(id,purpose,provider_name,model_id,status,input_chars,output_chars,duration_ms,error_code,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![run.id, run.purpose, run.provider_name, run.model_id, run.status, run.input_chars, run.output_chars, run.duration_ms, run.error_code, run.created_at]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn insert_payload_audit(&self, audit: &PayloadAudit) -> Result<(), String> {
        let connection = self.connection()?;
        connection.execute("INSERT INTO llm_payload_audits(id,llm_run_id,consent_type,document_id,chunk_id,purpose,sent_chars,total_document_chars,provider_name,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![audit.id, audit.llm_run_id, audit.consent_type, audit.document_id, audit.chunk_id, audit.purpose, audit.sent_chars, audit.total_document_chars, audit.provider_name, audit.created_at]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }

    pub fn delete_record(&self, id: &str) -> Result<(), String> {
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        for table in [
            "exam_point_segments",
            "exam_point_audio_ranges",
            "exam_point_source_refs",
        ] {
            transaction
                .execute(
                    &format!(
                        "DELETE FROM {table} WHERE point_id IN (SELECT id FROM exam_points WHERE record_id=?1)"
                    ),
                    params![id],
                )
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        transaction
            .execute("DELETE FROM exam_point_sets WHERE job_id=?1", params![id])
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute("DELETE FROM audio_jobs WHERE id=?1", params![id])
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute("DELETE FROM audio_records WHERE id=?1", params![id])
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .commit()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn list_lexicons(&self) -> Result<Vec<LexiconSummary>, String> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT id,name,textbook_title,version,terminology_count,chapter_count,updated_at,status FROM lexicons WHERE deleted_at IS NULL ORDER BY updated_at DESC").map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(LexiconSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    textbook_title: row.get(2)?,
                    version: row.get(3)?,
                    terminology_count: row.get(4)?,
                    chapter_count: row.get(5)?,
                    updated_at: row.get(6)?,
                    status: row.get(7)?,
                })
            })
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn get_lexicon(&self, id: &str) -> Result<Option<LexiconSummary>, String> {
        self.list_lexicons()
            .map(|list| list.into_iter().find(|lexicon| lexicon.id == id))
    }

    pub fn insert_lexicon(
        &self,
        profile: &LexiconProfile,
        managed_path: &str,
        file_type: &str,
        extracted_chars: i64,
        extraction_quality: &str,
        source_chunks: &[(String, String)],
    ) -> Result<(), String> {
        let connection = self.connection()?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let now = Utc::now().to_rfc3339();
        let profile_json =
            serde_json::to_string(profile).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction.execute("INSERT INTO source_documents(id,file_name,file_type,managed_path,extracted_chars,extraction_quality,metadata_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8)", params![profile.source_document_id, profile.textbook_title, file_type, managed_path, extracted_chars, extraction_quality, serde_json::json!({"title": profile.textbook_title}) .to_string(), now]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        for (ordinal, (label, text)) in source_chunks.iter().enumerate() {
            transaction.execute("INSERT INTO source_chunks(id,document_id,ordinal,source_label,text,char_count) VALUES(?1,?2,?3,?4,?5,?6)", params![uuid::Uuid::new_v4().to_string(), profile.source_document_id, ordinal as i64, label, text, text.chars().count() as i64]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        transaction.execute("INSERT INTO lexicons(id,name,textbook_title,version,terminology_count,chapter_count,status,profile_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9)", params![profile.id, profile.name, profile.textbook_title, profile.version, profile.terms.len() as i64, profile.chapters.len() as i64, "ready", profile_json, now]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction.execute("INSERT INTO lexicon_versions(id,lexicon_id,version,profile_json,created_at) VALUES(?1,?2,?3,?4,?5)", params![Uuid::new_v4().to_string(), profile.id, profile.version, &profile_json, now]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        for chapter in &profile.chapters {
            transaction.execute("INSERT INTO chapter_nodes(id,lexicon_id,parent_id,ordinal,title,label,source_document_id,source_page,source_slide) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![chapter.id, profile.id, chapter.parent_id, chapter.order, chapter.title, chapter.label, chapter.source_document_id, chapter.source_page, chapter.source_slide]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        for term in &profile.terms {
            transaction.execute("INSERT INTO lexicon_terms(id,lexicon_id,canonical_term,aliases_json,abbreviation,english_name,definition,chapter_ids_json,common_asr_errors_json,source_references_json,confirmed_by_user) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", params![term.id, profile.id, term.canonical_term, serde_json::to_string(&term.aliases).unwrap_or_else(|_| "[]".to_string()), term.abbreviation, term.english_name, term.definition, serde_json::to_string(&term.chapter_ids).unwrap_or_else(|_| "[]".to_string()), serde_json::to_string(&term.common_asr_errors).unwrap_or_else(|_| "[]".to_string()), serde_json::to_string(&term.source_references).unwrap_or_else(|_| "[]".to_string()), term.confirmed_by_user as i64]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        for rule in &profile.correction_rules {
            transaction.execute("INSERT INTO correction_rules(id,lexicon_id,original_text,corrected_text,enabled,created_by) VALUES(?1,?2,?3,?4,?5,?6)", params![rule.id, profile.id, rule.original_text, rule.corrected_text, rule.enabled as i64, rule.created_by]).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        transaction
            .commit()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn get_lexicon_profile(&self, id: &str) -> Result<Option<LexiconProfile>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT profile_json FROM lexicons WHERE id=?1 AND deleted_at IS NULL",
                params![id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
            .map(|value| {
                serde_json::from_str(&value).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
            })
            .transpose()
    }

    pub fn get_lexicon_profile_version(
        &self,
        id: &str,
        version: i64,
    ) -> Result<Option<LexiconProfile>, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT profile_json FROM lexicon_versions WHERE lexicon_id=?1 AND version=?2",
                params![id, version],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
            .map(|value| {
                serde_json::from_str(&value).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
            })
            .transpose()
    }

    pub fn source_chunks(&self, document_id: &str) -> Result<Vec<LexiconSourceChunk>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id,ordinal,source_label,text,char_count,selected_for_upload FROM source_chunks WHERE document_id=?1 ORDER BY ordinal",
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let rows = statement
            .query_map(params![document_id], |row| {
                Ok(LexiconSourceChunk {
                    id: row.get(0)?,
                    ordinal: row.get(1)?,
                    source_label: row.get(2)?,
                    text: row.get(3)?,
                    char_count: row.get(4)?,
                    selected_for_upload: row.get::<_, i64>(5)? != 0,
                })
            })
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn source_document_char_count(&self, document_id: &str) -> Result<usize, String> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT extracted_chars FROM source_documents WHERE id=?1",
                params![document_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?
            .map(|value| value.max(0) as usize)
            .ok_or_else(|| "LEXICON_NOT_FOUND".to_string())
    }

    pub fn mark_source_chunks_selected(
        &self,
        document_id: &str,
        selected_ids: &[String],
    ) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE source_chunks SET selected_for_upload=0 WHERE document_id=?1",
                params![document_id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        for chunk_id in selected_ids {
            connection
                .execute(
                    "UPDATE source_chunks SET selected_for_upload=1 WHERE id=?1 AND document_id=?2",
                    params![chunk_id, document_id],
                )
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        Ok(())
    }

    pub fn update_lexicon_profile(&self, profile: &LexiconProfile) -> Result<(), String> {
        let connection = self.connection()?;
        let transaction = connection
            .unchecked_transaction()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let now = Utc::now().to_rfc3339();
        let profile_json =
            serde_json::to_string(profile).map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "INSERT INTO lexicon_versions(id,lexicon_id,version,profile_json,created_at) VALUES(?1,?2,?3,?4,?5)",
                params![Uuid::new_v4().to_string(), profile.id, profile.version, &profile_json, now],
            )
            .map_err(|error| {
                if error.to_string().contains("UNIQUE") {
                    "LEXICON_VERSION_NOT_FOUND".to_string()
                } else {
                    "DATABASE_OPERATION_FAILED".to_string()
                }
            })?;
        transaction
            .execute(
                "UPDATE lexicons SET name=?1,textbook_title=?2,version=?3,terminology_count=?4,chapter_count=?5,status='ready',profile_json=?6,updated_at=?7 WHERE id=?8 AND deleted_at IS NULL",
                params![
                    profile.name,
                    profile.textbook_title,
                    profile.version,
                    profile.terms.len() as i64,
                    profile.chapters.len() as i64,
                    &profile_json,
                    now,
                    profile.id
                ],
            )
            .map_err(|_| "LEXICON_NOT_FOUND".to_string())?;
        transaction
            .execute(
                "DELETE FROM chapter_nodes WHERE lexicon_id=?1",
                params![profile.id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "DELETE FROM lexicon_terms WHERE lexicon_id=?1",
                params![profile.id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        transaction
            .execute(
                "DELETE FROM correction_rules WHERE lexicon_id=?1",
                params![profile.id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        insert_lexicon_children(&transaction, profile)?;
        transaction
            .commit()
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())
    }

    pub fn soft_delete_lexicon(&self, id: &str) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE lexicons SET deleted_at=?1,updated_at=?1 WHERE id=?2",
                params![Utc::now().to_rfc3339(), id],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        Ok(())
    }
}

fn insert_lexicon_children(
    transaction: &rusqlite::Transaction<'_>,
    profile: &LexiconProfile,
) -> Result<(), String> {
    for chapter in &profile.chapters {
        transaction
            .execute(
                "INSERT INTO chapter_nodes(id,lexicon_id,parent_id,ordinal,title,label,source_document_id,source_page,source_slide) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    chapter.id,
                    profile.id,
                    chapter.parent_id,
                    chapter.order,
                    chapter.title,
                    chapter.label,
                    chapter.source_document_id,
                    chapter.source_page,
                    chapter.source_slide
                ],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    }
    for term in &profile.terms {
        transaction
            .execute(
                "INSERT INTO lexicon_terms(id,lexicon_id,canonical_term,aliases_json,abbreviation,english_name,definition,chapter_ids_json,common_asr_errors_json,source_references_json,confirmed_by_user) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    term.id,
                    profile.id,
                    term.canonical_term,
                    serde_json::to_string(&term.aliases).unwrap_or_else(|_| "[]".to_string()),
                    term.abbreviation,
                    term.english_name,
                    term.definition,
                    serde_json::to_string(&term.chapter_ids).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&term.common_asr_errors)
                        .unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&term.source_references)
                        .unwrap_or_else(|_| "[]".to_string()),
                    term.confirmed_by_user as i64
                ],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    }
    for rule in &profile.correction_rules {
        transaction
            .execute(
                "INSERT INTO correction_rules(id,lexicon_id,original_text,corrected_text,enabled,created_by) VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    rule.id,
                    profile.id,
                    rule.original_text,
                    rule.corrected_text,
                    rule.enabled as i64,
                    rule.created_by
                ],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    }
    Ok(())
}

fn write_normalized_exam_points(
    transaction: &rusqlite::Transaction<'_>,
    job_id: &str,
    transcript_version_id: Option<&str>,
    lexicon_id: Option<&str>,
    lexicon_version: Option<i64>,
    points: &[ExamPoint],
    created_at: &str,
) -> Result<(), String> {
    if points.is_empty() {
        return Ok(());
    }

    let point_set_id = Uuid::new_v4().to_string();
    transaction
        .execute(
            "INSERT INTO exam_point_sets(id,job_id,transcript_version_id,lexicon_id,lexicon_version,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                point_set_id,
                job_id,
                transcript_version_id,
                lexicon_id,
                lexicon_version,
                created_at
            ],
        )
        .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;

    let mut chapter_keys: Vec<(Option<String>, String)> = Vec::new();
    for point in points {
        let chapter_key = (point.chapter_id.clone(), point.chapter_title.clone());
        if chapter_keys.iter().any(|existing| existing == &chapter_key) {
            continue;
        }
        let ordinal = chapter_keys.len() as i64;
        transaction
            .execute(
                "INSERT INTO exam_chapters(id,point_set_id,chapter_id,chapter_title,ordinal) VALUES(?1,?2,?3,?4,?5)",
                params![
                    Uuid::new_v4().to_string(),
                    point_set_id,
                    chapter_key.0,
                    chapter_key.1,
                    ordinal
                ],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        chapter_keys.push(chapter_key);
    }

    for point in points {
        for segment_id in &point.segment_ids {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO exam_point_segments(point_id,segment_id) VALUES(?1,?2)",
                    params![point.id, segment_id],
                )
                .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        }
        transaction
            .execute(
                "INSERT INTO exam_point_audio_ranges(point_id,start_ms,end_ms) VALUES(?1,?2,?3)",
                params![point.id, point.start_ms, point.end_ms],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
        let source_ref = serde_json::json!({
            "kind": &point.kind,
            "chapterId": &point.chapter_id,
            "segmentIds": &point.segment_ids,
        })
        .to_string();
        transaction
            .execute(
                "INSERT INTO exam_point_source_refs(point_id,source_ref_json) VALUES(?1,?2)",
                params![point.id, source_ref],
            )
            .map_err(|_| "DATABASE_OPERATION_FAILED".to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_audio_job_and_legacy_record_are_written_together() {
        let path = std::env::temp_dir().join(format!("verilecture-db-{}.sqlite", Uuid::new_v4()));
        let database = init_database(&path).expect("database should initialize");
        let record = ImportedRecord {
            id: Uuid::new_v4().to_string(),
            title: "test".to_string(),
            source_path: None,
            audio_path: None,
            created_at: Utc::now().to_rfc3339(),
            duration_ms: 1_000,
            status: "completed".to_string(),
            model_id: "fun-asr-nano-2512".to_string(),
            provider_name: None,
            lexicon_id: None,
            lexicon_version: None,
            language: "zh".to_string(),
            raw_segments: vec![TranscriptSegment {
                id: "raw-1".to_string(),
                start_ms: 0,
                end_ms: 1_000,
                text: "test".to_string(),
                language: "zh".to_string(),
                source: "raw".to_string(),
            }],
            calibrated_segments: vec![TranscriptSegment {
                id: "cal-raw-1".to_string(),
                start_ms: 0,
                end_ms: 1_000,
                text: "test".to_string(),
                language: "zh".to_string(),
                source: "calibrated".to_string(),
            }],
            exam_points: vec![ExamPoint {
                id: "point-1".to_string(),
                chapter_id: None,
                chapter_title: "UNMATCHED".to_string(),
                title: "Point".to_string(),
                detail: "Detail".to_string(),
                kind: "explicit".to_string(),
                segment_ids: vec!["raw-1".to_string()],
                start_ms: 0,
                end_ms: 1_000,
            }],
        };
        database
            .insert_record(&record)
            .expect("record should insert");
        let detail = database
            .get_record(&record.id)
            .expect("record should load")
            .expect("record exists");
        assert_eq!(detail.exam_points[0].kind, "explicit");
        let connection = Connection::open(&path).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM audio_jobs WHERE id=?1",
                params![record.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        let normalized_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM exam_point_sets WHERE job_id=?1",
                params![record.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(normalized_count, 1);
        let relation_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM exam_point_segments WHERE point_id=?1",
                params!["point-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(relation_count, 1);
        drop(connection);
        database
            .delete_record(&record.id)
            .expect("record should delete");
        let connection = Connection::open(&path).unwrap();
        let orphan_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM exam_point_segments WHERE point_id=?1",
                params!["point-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(orphan_count, 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn lexicon_update_keeps_the_previous_profile_version() {
        let path =
            std::env::temp_dir().join(format!("verilecture-lexicon-{}.sqlite", Uuid::new_v4()));
        let database = init_database(&path).expect("database should initialize");
        let now = Utc::now().to_rfc3339();
        let profile = LexiconProfile {
            id: Uuid::new_v4().to_string(),
            name: "Computer Networks".to_string(),
            version: 1,
            textbook_title: "Computer Networks".to_string(),
            source_document_id: Uuid::new_v4().to_string(),
            chapters: Vec::new(),
            terms: vec![LexiconTerm {
                id: Uuid::new_v4().to_string(),
                canonical_term: "TCP".to_string(),
                aliases: vec!["传输控制协议".to_string()],
                abbreviation: Some("TCP".to_string()),
                english_name: Some("Transmission Control Protocol".to_string()),
                definition: Some("Reliable transport protocol".to_string()),
                chapter_ids: Vec::new(),
                common_asr_errors: Vec::new(),
                source_references: vec!["p.1".to_string()],
                confirmed_by_user: false,
            }],
            correction_rules: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        };
        database
            .insert_lexicon(&profile, "local/source.txt", "txt", 40, "good", &[])
            .expect("profile should insert");
        let mut next = profile.clone();
        next.version = 2;
        next.name = "Computer Networks v2".to_string();
        next.terms[0].confirmed_by_user = true;
        next.updated_at = Utc::now().to_rfc3339();
        database
            .update_lexicon_profile(&next)
            .expect("new version should insert");
        assert_eq!(
            database
                .get_lexicon_profile_version(&profile.id, 1)
                .unwrap()
                .unwrap()
                .name,
            "Computer Networks"
        );
        assert_eq!(
            database
                .get_lexicon_profile(&profile.id)
                .unwrap()
                .unwrap()
                .version,
            2
        );
        let _ = std::fs::remove_file(path);
    }
}
