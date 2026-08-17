export type Locale = "zh-CN" | "en-US";
export type ViewId = "import" | "records" | "lexicons" | "settings" | "about";
export type ModelId = "qwen3-asr-1.7b" | "qwen3-asr-0.6b" | "fun-asr-nano-2512";
export type ModelStatus =
  | "not_installed"
  | "checking"
  | "downloading"
  | "paused"
  | "verifying"
  | "installing"
  | "testing"
  | "ready"
  | "error"
  | "cancelled"
  | "corrupted";
export type ProcessingMode = "local";
export type AudioLanguage = "auto" | "zh" | "en" | "yue";

export interface HardwareProfile {
  os: string;
  osVersion: string;
  architecture: string;
  cpuName: string;
  logicalCores: number;
  avx2: boolean;
  totalRamBytes: number;
  availableRamBytes: number;
  diskFreeBytes: number;
  nvidiaDetected: boolean;
  gpuName: string | null;
  vramBytes: number | null;
  driverVersion: string | null;
  nvidiaSmi: boolean;
  cudaDriverApi: boolean;
  cudaSmokeTest: boolean;
  networkAvailable: boolean;
  proxyConfigured: boolean;
  proxySource: string | null;
  modelDirectoryWritable: boolean;
  scannedAt: string;
}

export interface ModelOption {
  id: ModelId;
  name: string;
  description: string;
  runtime: string;
  downloadBytes: number;
  diskBytes: number;
  requiresCuda: boolean;
  requiresAligner: boolean;
  status: ModelStatus;
  supported: boolean;
  recommended: boolean;
  reason: string;
}

export interface ProviderConfig {
  provider: string;
  protocol:
    | "openai_compatible"
    | "openai_responses"
    | "anthropic_messages"
    | "gemini_generate_content";
  baseUrl: string;
  modelId: string;
  organization?: string;
  timeoutSeconds: number;
  configured: boolean;
  tested: boolean;
  consentGranted: boolean;
  secretRef?: string;
}

export interface TranscriptSegment {
  id: string;
  startMs: number;
  endMs: number;
  text: string;
  language: string;
  source: "raw" | "calibrated";
}

export interface ExamPoint {
  id: string;
  chapterId: string | null;
  chapterTitle: string;
  title: string;
  detail: string;
  segmentIds: string[];
  startMs: number;
  endMs: number;
}

export interface RecordSummary {
  id: string;
  title: string;
  createdAt: string;
  durationMs: number;
  status: "completed" | "processing" | "failed" | "cancelled";
  modelId: ModelId;
  providerName: string | null;
  lexiconName: string | null;
  sourcePath: string | null;
}

export interface RecordDetail extends RecordSummary {
  audioPath: string | null;
  lexiconId?: string | null;
  language: AudioLanguage;
  rawSegments: TranscriptSegment[];
  calibratedSegments: TranscriptSegment[];
  examPoints: ExamPoint[];
}

export interface LexiconSummary {
  id: string;
  name: string;
  textbookTitle: string;
  version: number;
  terminologyCount: number;
  chapterCount: number;
  updatedAt: string;
  status: "ready" | "parsing" | "error";
}

export interface LexiconChapter {
  id: string;
  parentId: string | null;
  order: number;
  title: string;
  label: string | null;
  sourceDocumentId: string;
  sourcePage: number | null;
  sourceSlide: number | null;
}

export interface LexiconTerm {
  id: string;
  canonicalTerm: string;
  aliases: string[];
  abbreviation: string | null;
  englishName: string | null;
  definition: string | null;
  chapterIds: string[];
  commonAsrErrors: string[];
  sourceReferences: string[];
  confirmedByUser: boolean;
}

export interface CorrectionRule {
  id: string;
  originalText: string;
  correctedText: string;
  enabled: boolean;
  createdBy: "user" | "lexicon" | string;
}

export interface LexiconProfile {
  id: string;
  name: string;
  version: number;
  textbookTitle: string;
  sourceDocumentId: string;
  chapters: LexiconChapter[];
  terms: LexiconTerm[];
  correctionRules: CorrectionRule[];
  createdAt: string;
  updatedAt: string;
}

export interface AppSnapshot {
  onboardingComplete: boolean;
  locale: Locale;
  processingMode: ProcessingMode;
  selectedModelId: ModelId | null;
  hardware: HardwareProfile | null;
  models: ModelOption[];
  provider: ProviderConfig | null;
  modelDirectory: string;
  records: RecordSummary[];
  lexicons: LexiconSummary[];
}

export interface InstallProgress {
  modelId: ModelId;
  stage:
    | "checking"
    | "downloading"
    | "paused"
    | "verifying"
    | "installing"
    | "testing"
    | "smoke_test"
    | "ready"
    | "error";
  fileName: string;
  downloadedBytes: number;
  totalBytes: number;
  speedBytesPerSecond: number;
  message: string;
}
