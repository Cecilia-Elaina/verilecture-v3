PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS hardware_profiles (
  id TEXT PRIMARY KEY NOT NULL,
  profile_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS model_registry_snapshots (
  id TEXT PRIMARY KEY NOT NULL,
  registry_version TEXT NOT NULL,
  registry_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS installed_model_profiles (
  id TEXT PRIMARY KEY NOT NULL,
  model_id TEXT NOT NULL,
  registry_version TEXT NOT NULL,
  install_path TEXT NOT NULL,
  status TEXT NOT NULL,
  manifest_json TEXT NOT NULL,
  last_verified_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS model_install_events (
  id TEXT PRIMARY KEY NOT NULL,
  model_id TEXT NOT NULL,
  stage TEXT NOT NULL,
  file_name TEXT,
  downloaded_bytes INTEGER NOT NULL DEFAULT 0,
  total_bytes INTEGER NOT NULL DEFAULT 0,
  error_code TEXT,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS provider_configs (
  id TEXT PRIMARY KEY NOT NULL,
  provider TEXT NOT NULL,
  protocol TEXT NOT NULL,
  base_url TEXT NOT NULL,
  model_id TEXT NOT NULL,
  organization TEXT,
  timeout_seconds INTEGER NOT NULL DEFAULT 60,
  tested INTEGER NOT NULL DEFAULT 0,
  secret_ref TEXT,
  consent_granted INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS privacy_consents (
  consent_type TEXT PRIMARY KEY NOT NULL,
  granted INTEGER NOT NULL DEFAULT 0,
  provider_id TEXT,
  granted_at TEXT,
  revoked_at TEXT,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS lexicons (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  textbook_title TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1,
  terminology_count INTEGER NOT NULL DEFAULT 0,
  chapter_count INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'ready',
  profile_json TEXT NOT NULL DEFAULT '{}',
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS lexicon_versions (
  id TEXT PRIMARY KEY NOT NULL,
  lexicon_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  profile_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (lexicon_id, version),
  FOREIGN KEY (lexicon_id) REFERENCES lexicons(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS source_documents (
  id TEXT PRIMARY KEY NOT NULL,
  file_name TEXT NOT NULL,
  file_type TEXT NOT NULL,
  managed_path TEXT NOT NULL,
  extracted_chars INTEGER NOT NULL DEFAULT 0,
  extraction_quality TEXT NOT NULL DEFAULT 'unknown',
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS source_chunks (
  id TEXT PRIMARY KEY NOT NULL,
  document_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  source_label TEXT,
  text TEXT NOT NULL,
  char_count INTEGER NOT NULL,
  selected_for_upload INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (document_id) REFERENCES source_documents(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS textbook_metadata (
  document_id TEXT PRIMARY KEY NOT NULL,
  title TEXT,
  edition TEXT,
  authors_json TEXT NOT NULL DEFAULT '[]',
  publisher TEXT,
  isbn TEXT,
  subject TEXT,
  language TEXT,
  FOREIGN KEY (document_id) REFERENCES source_documents(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS chapter_nodes (
  id TEXT PRIMARY KEY NOT NULL,
  lexicon_id TEXT NOT NULL,
  parent_id TEXT,
  ordinal INTEGER NOT NULL,
  title TEXT NOT NULL,
  label TEXT,
  source_document_id TEXT,
  source_page INTEGER,
  source_slide INTEGER,
  FOREIGN KEY (lexicon_id) REFERENCES lexicons(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS lexicon_terms (
  id TEXT PRIMARY KEY NOT NULL,
  lexicon_id TEXT NOT NULL,
  canonical_term TEXT NOT NULL,
  aliases_json TEXT NOT NULL DEFAULT '[]',
  abbreviation TEXT,
  english_name TEXT,
  definition TEXT,
  chapter_ids_json TEXT NOT NULL DEFAULT '[]',
  common_asr_errors_json TEXT NOT NULL DEFAULT '[]',
  source_references_json TEXT NOT NULL DEFAULT '[]',
  confirmed_by_user INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (lexicon_id) REFERENCES lexicons(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS correction_rules (
  id TEXT PRIMARY KEY NOT NULL,
  lexicon_id TEXT NOT NULL,
  original_text TEXT NOT NULL,
  corrected_text TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  created_by TEXT NOT NULL,
  FOREIGN KEY (lexicon_id) REFERENCES lexicons(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS llm_runs (
  id TEXT PRIMARY KEY NOT NULL,
  purpose TEXT NOT NULL,
  provider_name TEXT NOT NULL,
  model_id TEXT NOT NULL,
  status TEXT NOT NULL,
  input_chars INTEGER NOT NULL DEFAULT 0,
  output_chars INTEGER NOT NULL DEFAULT 0,
  duration_ms INTEGER NOT NULL DEFAULT 0,
  error_code TEXT,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS llm_payload_audits (
  id TEXT PRIMARY KEY NOT NULL,
  llm_run_id TEXT,
  consent_type TEXT NOT NULL,
  document_id TEXT,
  chunk_id TEXT,
  purpose TEXT NOT NULL,
  sent_chars INTEGER NOT NULL DEFAULT 0,
  total_document_chars INTEGER,
  provider_name TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS audio_jobs (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  source_filename TEXT NOT NULL,
  managed_audio_path TEXT,
  duration_ms INTEGER NOT NULL DEFAULT 0,
  file_size_bytes INTEGER NOT NULL DEFAULT 0,
  audio_format TEXT,
  language_preference TEXT NOT NULL,
  asr_profile_id TEXT NOT NULL,
  lexicon_profile_id TEXT,
  lexicon_version INTEGER,
  provider_config_id TEXT,
  current_stage TEXT NOT NULL,
  progress_percent INTEGER NOT NULL DEFAULT 0,
  error_code TEXT,
  error_message_safe TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT
);
CREATE TABLE IF NOT EXISTS audio_assets (
  id TEXT PRIMARY KEY NOT NULL,
  job_id TEXT NOT NULL,
  source_path_display TEXT,
  managed_path TEXT NOT NULL,
  sha256 TEXT,
  is_original_copy INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  FOREIGN KEY (job_id) REFERENCES audio_jobs(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS job_events (
  id TEXT PRIMARY KEY NOT NULL,
  job_id TEXT NOT NULL,
  stage TEXT NOT NULL,
  progress_percent INTEGER NOT NULL DEFAULT 0,
  message_code TEXT,
  error_code TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (job_id) REFERENCES audio_jobs(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS transcript_versions (
  id TEXT PRIMARY KEY NOT NULL,
  job_id TEXT NOT NULL,
  version_type TEXT NOT NULL,
  parent_version_id TEXT,
  lexicon_id TEXT,
  lexicon_version INTEGER,
  created_at TEXT NOT NULL,
  FOREIGN KEY (job_id) REFERENCES audio_jobs(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS transcript_words (
  id TEXT PRIMARY KEY NOT NULL,
  transcript_version_id TEXT NOT NULL,
  segment_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  word TEXT NOT NULL,
  start_ms INTEGER,
  end_ms INTEGER,
  FOREIGN KEY (transcript_version_id) REFERENCES transcript_versions(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS audio_records (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  source_path TEXT,
  audio_path TEXT,
  created_at TEXT NOT NULL,
  duration_ms INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL,
  model_id TEXT NOT NULL,
  provider_name TEXT,
  lexicon_id TEXT,
  lexicon_version INTEGER,
  language TEXT NOT NULL,
  error_code TEXT,
  error_message_safe TEXT,
  FOREIGN KEY (lexicon_id) REFERENCES lexicons(id)
);
CREATE TABLE IF NOT EXISTS transcript_segments (
  id TEXT PRIMARY KEY NOT NULL,
  record_id TEXT NOT NULL,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL,
  text TEXT NOT NULL,
  language TEXT NOT NULL,
  source TEXT NOT NULL,
  FOREIGN KEY (record_id) REFERENCES audio_records(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS exam_points (
  id TEXT PRIMARY KEY NOT NULL,
  record_id TEXT NOT NULL,
  chapter_id TEXT,
  chapter_title TEXT NOT NULL,
  title TEXT NOT NULL,
  detail TEXT NOT NULL,
  kind TEXT NOT NULL DEFAULT 'inferred',
  segment_ids_json TEXT NOT NULL,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL,
  FOREIGN KEY (record_id) REFERENCES audio_records(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS exam_point_sets (
  id TEXT PRIMARY KEY NOT NULL,
  job_id TEXT NOT NULL,
  transcript_version_id TEXT,
  lexicon_id TEXT,
  lexicon_version INTEGER,
  created_at TEXT NOT NULL,
  FOREIGN KEY (job_id) REFERENCES audio_jobs(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS exam_chapters (
  id TEXT PRIMARY KEY NOT NULL,
  point_set_id TEXT NOT NULL,
  chapter_id TEXT,
  chapter_title TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  FOREIGN KEY (point_set_id) REFERENCES exam_point_sets(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS exam_point_segments (
  point_id TEXT NOT NULL,
  segment_id TEXT NOT NULL,
  PRIMARY KEY (point_id, segment_id)
);
CREATE TABLE IF NOT EXISTS exam_point_audio_ranges (
  point_id TEXT NOT NULL,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS exam_point_source_refs (
  point_id TEXT NOT NULL,
  source_ref_json TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audio_records_created_at ON audio_records(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_transcript_segments_record_id ON transcript_segments(record_id, start_ms);
