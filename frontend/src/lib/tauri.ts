import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  AppSnapshot,
  AudioLanguage,
  InstallProgress,
  ModelId,
  ModelOption,
  ProviderConfig,
  RecordDetail,
  RecordSummary,
  HardwareProfile,
  LexiconProfile,
  LexiconSummary,
} from "./contracts";

export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri) throw new Error("TAURI_UNAVAILABLE");
  return invoke<T>(command, args);
}

export async function scanHardware(): Promise<HardwareProfile> {
  return call<HardwareProfile>("scan_hardware");
}

export async function getModelCatalog(): Promise<ModelOption[]> {
  return call<ModelOption[]>("get_model_catalog");
}

export async function getSnapshot(): Promise<AppSnapshot> {
  return call<AppSnapshot>("get_app_snapshot");
}

export async function installModel(modelId: ModelId): Promise<void> {
  await call<void>("install_model", { modelId });
}

export async function pauseModel(modelId: ModelId): Promise<void> {
  await call<void>("pause_model_download", { modelId });
}

export async function resumeModel(modelId: ModelId): Promise<void> {
  await call<void>("resume_model_download", { modelId });
}

export async function cancelModel(modelId: ModelId): Promise<void> {
  await call<void>("cancel_model_download", { modelId });
}

export async function cancelAudioJob(jobId: string): Promise<void> {
  await call<void>("cancel_audio_job", { jobId });
}

export async function selectModel(modelId: ModelId): Promise<void> {
  await call<void>("select_model", { modelId });
}

export async function verifyModel(modelId: ModelId): Promise<void> {
  await call<void>("verify_model", { modelId });
}

export async function completeOnboarding(): Promise<void> {
  await call<void>("complete_onboarding");
}

export async function testProvider(
  config: ProviderConfig,
  apiKey: string,
): Promise<{ ok: boolean; message: string }> {
  return call<{ ok: boolean; message: string }>("test_text_provider", {
    config,
    apiKey,
  });
}

export async function saveProvider(
  config: ProviderConfig,
  apiKey: string,
): Promise<void> {
  await call<void>("save_text_provider", { config, apiKey });
}

export async function importAudio(args: {
  path: string;
  title: string;
  language: AudioLanguage;
  lexiconId?: string | null;
  jobId?: string;
}): Promise<RecordDetail> {
  return call<RecordDetail>("import_audio", { request: args });
}

export async function pickFile(
  kind: "audio" | "textbook",
): Promise<string | null> {
  if (!isTauri) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    multiple: false,
    directory: false,
    filters:
      kind === "audio"
        ? [
            {
              name: "Audio",
              extensions: [
                "wav",
                "mp3",
                "m4a",
                "aac",
                "flac",
                "ogg",
                "mp4",
                "mkv",
                "webm",
              ],
            },
          ]
        : [
            {
              name: "Textbook",
              extensions: ["pdf", "docx", "pptx", "txt", "md", "markdown"],
            },
          ],
  });
  return typeof selected === "string" ? selected : null;
}

export async function importLexicon(
  path: string,
  name?: string,
): Promise<LexiconSummary> {
  return call<LexiconSummary>("import_lexicon", {
    request: { path, name: name || null },
  });
}

export async function deleteLexicon(id: string): Promise<void> {
  await call<void>("delete_lexicon", { id });
}

export async function getLexicon(id: string): Promise<LexiconProfile> {
  return call<LexiconProfile>("get_lexicon", { id });
}

export async function saveLexicon(
  profile: LexiconProfile,
): Promise<LexiconSummary> {
  return call<LexiconSummary>("update_lexicon", { profile });
}

export async function setPrivacyConsent(
  consentType:
    | "cloud_llm_transcript"
    | "cloud_llm_lexicon_structured_data"
    | "cloud_llm_textbook_excerpt",
  granted: boolean,
): Promise<void> {
  await call<void>("set_privacy_consent", { consentType, granted });
}

export async function generateLexicon(id: string): Promise<LexiconSummary> {
  return call<LexiconSummary>("generate_lexicon", { id });
}

export async function getLexiconUploadPreview(id: string): Promise<{
  lexiconId: string;
  selection: {
    totalDocumentChars: number;
    budgetChars: number;
    selectedChars: number;
    chunks: Array<{
      id: string;
      sourceLabel: string | null;
      text: string;
      charCount: number;
      selectedForUpload: boolean;
    }>;
  };
}> {
  return call("get_lexicon_upload_preview", { id });
}

export async function generateExamPoints(
  recordId: string,
): Promise<RecordDetail> {
  return call<RecordDetail>("generate_exam_points", { recordId });
}

export async function exportRecord(
  recordId: string,
  path: string,
  format: "json" | "md" | "txt",
): Promise<void> {
  await call<void>("export_record", {
    request: { id: recordId, path, format },
  });
}

export async function pickSaveFile(
  format: "json" | "md" | "txt",
): Promise<string | null> {
  if (!isTauri) return null;
  const { save } = await import("@tauri-apps/plugin-dialog");
  const selected = await save({
    filters: [
      {
        name:
          format === "json" ? "JSON" : format === "md" ? "Markdown" : "Text",
        extensions: [format],
      },
    ],
  });
  return selected;
}

export function audioAssetUrl(path: string | null): string | null {
  if (!path) return null;
  return isTauri ? convertFileSrc(path, "asset") : path;
}

export async function listRecords(): Promise<RecordSummary[]> {
  return call<RecordSummary[]>("list_records");
}

export async function getRecord(id: string): Promise<RecordDetail> {
  return call<RecordDetail>("get_record", { id });
}

export async function deleteRecord(
  id: string,
  deleteCopy: boolean,
): Promise<void> {
  await call<void>("delete_record", { id, deleteCopy });
}

export async function listLexicons(): Promise<LexiconSummary[]> {
  return call<LexiconSummary[]>("list_lexicons");
}

export async function subscribeInstallProgress(
  handler: (progress: InstallProgress) => void,
): Promise<() => void> {
  if (!isTauri) return () => undefined;
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen<InstallProgress>(
    "model-install-progress",
    (event) => handler(event.payload),
  );
  return unlisten;
}

export async function subscribeAudioProgress(
  handler: (progress: {
    jobId: string;
    stage: string;
    progressPercent: number;
    message: string;
  }) => void,
): Promise<() => void> {
  if (!isTauri) return () => undefined;
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen<{
    jobId: string;
    stage: string;
    progressPercent: number;
    message: string;
  }>("audio-job-progress", (event) => handler(event.payload));
  return unlisten;
}
