import { useEffect, useMemo, useRef, useState } from "react";
import type { Dispatch, ReactNode, SetStateAction } from "react";
import {
  ArrowLeft,
  ArrowRight,
  BookOpen,
  Check,
  ChevronDown,
  CircleHelp,
  Cpu,
  Download,
  FileAudio,
  FileText,
  FolderOpen,
  HardDrive,
  Headphones,
  Info,
  Languages,
  Library,
  LockKeyhole,
  Menu,
  MonitorCog,
  MoreHorizontal,
  Pause,
  Pencil,
  Play,
  Plus,
  RefreshCw,
  Save,
  Search,
  Settings2,
  ShieldCheck,
  Sparkles,
  Square,
  Trash2,
  Upload,
  X,
  Zap,
} from "lucide-react";
import { createTranslator } from "@/i18n/messages";
import type {
  AppSnapshot,
  AudioLanguage,
  ExamPoint,
  HardwareProfile,
  InstallProgress,
  LexiconProfile,
  LexiconTerm,
  LexiconSummary,
  Locale,
  ModelId,
  ModelOption,
  ProviderConfig,
  RecordDetail,
  RecordSummary,
  TranscriptSegment,
  ViewId,
} from "@/lib/contracts";
import {
  audioAssetUrl,
  call,
  cancelAudioJob,
  cancelModel,
  completeOnboarding,
  deleteLexicon,
  deleteRecord,
  exportRecord,
  generateLexicon,
  generateExamPoints,
  getModelCatalog,
  getLexicon,
  getRecord,
  getSnapshot,
  importAudio,
  importLexicon,
  installModel,
  isTauri,
  listLexicons,
  listRecords,
  pauseModel,
  pickFile,
  pickSaveFile,
  resumeModel,
  saveProvider,
  saveLexicon,
  scanHardware,
  setPrivacyConsent,
  selectModel,
  verifyModel,
  subscribeInstallProgress,
  subscribeAudioProgress,
  testProvider,
} from "@/lib/tauri";

const modelOrder: ModelId[] = [
  "qwen3-asr-1.7b",
  "qwen3-asr-0.6b",
  "fun-asr-nano-2512",
];

const providerOptions = [
  ["OpenAI", "providerOpenAI"],
  ["Anthropic", "providerAnthropic"],
  ["Google Gemini", "providerGemini"],
  ["DeepSeek", "providerDeepSeek"],
  ["阿里云百炼 / DashScope", "providerDashscope"],
  ["Moonshot / Kimi", "providerMoonshot"],
  ["智谱 AI / GLM", "providerZhipu"],
  ["MiniMax", "providerMinimax"],
  ["火山引擎方舟 / 豆包", "providerVolcengine"],
  ["SiliconFlow", "providerSiliconflow"],
  ["OpenRouter", "providerOpenrouter"],
  ["xAI", "providerXai"],
  ["Mistral", "providerMistral"],
  ["自定义 OpenAI-compatible", "providerCustom"],
] as const;

function providerBaseUrl(provider: string): string {
  return (
    {
      OpenAI: "https://api.openai.com/v1",
      Anthropic: "https://api.anthropic.com/v1",
      "Google Gemini": "https://generativelanguage.googleapis.com/v1beta",
      DeepSeek: "https://api.deepseek.com",
      "阿里云百炼 / DashScope":
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
      "Moonshot / Kimi": "https://api.moonshot.cn/v1",
      "智谱 AI / GLM": "https://open.bigmodel.cn/api/paas/v4",
      MiniMax: "https://api.minimax.io/v1",
      "火山引擎方舟 / 豆包": "https://ark.cn-beijing.volces.com/api/v3",
      SiliconFlow: "https://api.siliconflow.cn/v1",
      OpenRouter: "https://openrouter.ai/api/v1",
      xAI: "https://api.x.ai/v1",
      Mistral: "https://api.mistral.ai/v1",
    }[provider] ?? ""
  );
}

function errorCode(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message?: unknown }).message ?? "");
  }
  return String(error);
}

function analysisErrorMessage(
  error: unknown,
  t: ReturnType<typeof createTranslator>,
): string {
  const messages: Record<string, string> = {
    PROVIDER_REQUEST_REJECTED: "analysisProviderRequestRejected",
    PROVIDER_AUTH_FAILED: "analysisProviderAuthFailed",
    PROVIDER_BALANCE_INSUFFICIENT: "analysisProviderBalance",
    PROVIDER_ENDPOINT_OR_MODEL_NOT_FOUND: "analysisProviderNotFound",
    PROVIDER_RATE_LIMITED: "analysisProviderRateLimited",
    PROVIDER_TIMEOUT: "analysisProviderTimeout",
    PROVIDER_NETWORK_FAILED: "analysisProviderNetwork",
    PROVIDER_RESPONSE_NOT_JSON: "analysisProviderResponseNotJson",
    PROVIDER_EMPTY_RESPONSE: "analysisProviderEmpty",
    PROVIDER_OUTPUT_TRUNCATED: "analysisProviderTruncated",
    PROVIDER_CONTENT_FILTERED: "analysisProviderContentFiltered",
    PROVIDER_JSON_OUTPUT_INVALID: "analysisProviderJsonInvalid",
    PROVIDER_RESPONSE_INVALID: "analysisProviderResponseInvalid",
    LLM_SCHEMA_INVALID: "analysisProviderSchemaInvalid",
  };
  return t(messages[errorCode(error)] ?? "analysisFailed");
}

function demoHardware(locale: Locale = "zh-CN"): HardwareProfile {
  return {
    os: "Windows 10/11",
    osVersion: "Windows 10/11 (preview)",
    architecture: "x86_64",
    cpuName: locale === "zh-CN" ? "需先扫描硬件" : "Hardware scan required",
    logicalCores: 8,
    avx2: true,
    totalRamBytes: 16 * 1024 ** 3,
    availableRamBytes: 10 * 1024 ** 3,
    diskFreeBytes: 224 * 1024 ** 3,
    nvidiaDetected: false,
    gpuName: null,
    vramBytes: null,
    driverVersion: null,
    nvidiaSmi: false,
    cudaDriverApi: false,
    cudaSmokeTest: false,
    networkAvailable: true,
    proxyConfigured: false,
    proxySource: null,
    modelDirectoryWritable: true,
    scannedAt: new Date().toISOString(),
  };
}

function demoModels(locale: Locale = "zh-CN"): ModelOption[] {
  const isZh = locale === "zh-CN";
  return [
    {
      id: "qwen3-asr-1.7b",
      name: "Qwen3-ASR-1.7B",
      description: isZh
        ? "本地识别质量更高，提供 Forced Aligner 时间戳"
        : "Higher-quality local recognition with Forced Aligner timestamps",
      runtime: "NVIDIA CUDA",
      downloadBytes: 6_543_068_817,
      diskBytes: 7_500_000_000,
      requiresCuda: true,
      requiresAligner: true,
      status: "not_installed",
      supported: false,
      recommended: false,
      reason: isZh ? "请先完成硬件扫描" : "Scan hardware first",
    },
    {
      id: "qwen3-asr-0.6b",
      name: "Qwen3-ASR-0.6B",
      description: isZh
        ? "显存占用较低，提供 Forced Aligner 时间戳"
        : "Lower VRAM use with Forced Aligner timestamps",
      runtime: "NVIDIA CUDA",
      downloadBytes: 3_720_574_187,
      diskBytes: 4_300_000_000,
      requiresCuda: true,
      requiresAligner: true,
      status: "not_installed",
      supported: false,
      recommended: false,
      reason: isZh ? "请先完成硬件扫描" : "Scan hardware first",
    },
    {
      id: "fun-asr-nano-2512",
      name: "Fun-ASR-Nano-2512",
      description: isZh
        ? "没有可用 NVIDIA CUDA 时使用 CPU 本地转写"
        : "CPU local transcription for machines without usable NVIDIA CUDA",
      runtime: "CPU",
      downloadBytes: 1_280_490_277,
      diskBytes: 1_600_000_000,
      requiresCuda: false,
      requiresAligner: false,
      status: "not_installed",
      supported: true,
      recommended: true,
      reason: isZh
        ? "支持 CPU 转写，速度通常较慢"
        : "CPU transcription is supported but usually slower",
    },
  ];
}

const emptySnapshot: AppSnapshot = {
  onboardingComplete: false,
  locale: "zh-CN",
  processingMode: "local",
  selectedModelId: null,
  hardware: null,
  models: demoModels(),
  provider: null,
  modelDirectory: "正在读取…",
  records: [],
  lexicons: [],
};

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return "—";
  const units = ["B", "KB", "MB", "GB"];
  const index = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  return `${(bytes / 1024 ** index).toFixed(index > 1 ? 1 : 0)} ${units[index]}`;
}

function formatDuration(ms: number) {
  const totalSeconds = Math.max(0, Math.round(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function audioProgressMessage(message: string, t: (key: string) => string) {
  const messages: Record<string, string> = {
    AUDIO_COPYING: t("audioCopying"),
    AUDIO_DECODING: t("audioDecoding"),
    AUDIO_VAD: t("audioVad"),
    AUDIO_TRANSCRIBING: t("audioTranscribing"),
    AUDIO_CALIBRATING: t("audioCalibrating"),
    AUDIO_IMPORT_COMPLETED: t("statusCompleted"),
    JOB_CANCELLED: t("statusCancelled"),
  };
  return messages[message] ?? t("processing");
}

function modelProgressStage(
  stage: InstallProgress["stage"],
  t: (key: string) => string,
) {
  const messages: Partial<Record<InstallProgress["stage"], string>> = {
    checking: t("modelStatusChecking"),
    downloading: t("modelStatusDownloading"),
    paused: t("modelStatusPaused"),
    verifying: t("modelStatusVerifying"),
    installing: t("modelStatusInstalling"),
    testing: t("modelStatusTesting"),
    smoke_test: t("modelStatusTesting"),
    ready: t("modelStatusReady"),
    error: t("modelStatusError"),
  };
  return messages[stage] ?? t("processing");
}

function modelProgressMessage(message: string, t: (key: string) => string) {
  const messages: Record<string, string> = {
    Downloading: t("modelStatusDownloading"),
    "Verifying checksum": t("modelStatusVerifying"),
    "Installing CUDA runtime": t("modelRuntimeInstalling"),
    "CUDA runtime ready": t("modelRuntimeReady"),
    "Loading model and running smoke test": t("modelStatusTesting"),
    READY: t("modelStatusReady"),
    MODEL_DOWNLOAD_FAILED: t("installFailed"),
    MODEL_DOWNLOAD_CANCELLED: t("installCancelled"),
  };
  return (
    messages[message] ??
    (message.startsWith("MODEL_") ? t("installFailed") : t("processing"))
  );
}

function modelInstallErrorMessage(error: unknown, t: (key: string) => string) {
  const message = error instanceof Error ? error.message : "";
  if (message === "MODEL_RUNTIME_SOURCE_UNAVAILABLE") {
    return t("runtimeSourceUnavailable");
  }
  if (message === "MODEL_DOWNLOAD_CANCELLED") {
    return t("installCancelled");
  }
  if (message === "MODEL_DOWNLOAD_FAILED") {
    return t("installFailed");
  }
  return t("errorGeneric");
}

function lexiconStatusLabel(status: string, t: (key: string) => string) {
  if (status === "ready") return t("lexiconStatusReady");
  if (status === "failed" || status === "error") return t("statusFailed");
  return t("processing");
}

function formatDate(value: string, locale: Locale) {
  try {
    return new Intl.DateTimeFormat(locale, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(value));
  } catch {
    return value;
  }
}

function modelLabel(id: ModelId | null, models: ModelOption[]) {
  return models.find((model) => model.id === id)?.name ?? id ?? "—";
}

function makeDemoRecord(
  fileName: string,
  language: AudioLanguage,
  modelId: ModelId,
): RecordDetail {
  const now = new Date().toISOString();
  const rawSegments: TranscriptSegment[] = [
    {
      id: "demo-segment-1",
      startMs: 0,
      endMs: 7800,
      text: "今天先看计算机网络中的分组交换。",
      language: language === "auto" ? "zh" : language,
      source: "raw",
    },
    {
      id: "demo-segment-2",
      startMs: 8200,
      endMs: 16400,
      text: "运输层负责端到端的可靠传输和拥塞控制。",
      language: language === "auto" ? "zh" : language,
      source: "raw",
    },
    {
      id: "demo-segment-3",
      startMs: 17100,
      endMs: 25100,
      text: "重点比较 TCP 和 UDP 的服务特点与适用场景。",
      language: language === "auto" ? "zh" : language,
      source: "raw",
    },
  ];
  const calibratedSegments = rawSegments.map((segment) => ({
    ...segment,
    source: "calibrated" as const,
  }));
  const examPoints: ExamPoint[] = [
    {
      id: "demo-point-1",
      chapterId: null,
      chapterTitle: "未匹配章节",
      title: "分组交换的基本过程",
      detail:
        "理解分组交换中分组、排队和转发的基本过程，以及它与电路交换的差异。",
      segmentIds: ["demo-segment-1"],
      startMs: 0,
      endMs: 7800,
    },
    {
      id: "demo-point-2",
      chapterId: null,
      chapterTitle: "未匹配章节",
      title: "运输层的职责",
      detail: "掌握端到端可靠传输、流量控制和拥塞控制的作用。",
      segmentIds: ["demo-segment-2"],
      startMs: 8200,
      endMs: 16400,
    },
    {
      id: "demo-point-3",
      chapterId: null,
      chapterTitle: "未匹配章节",
      title: "TCP 与 UDP 的区别",
      detail: "比较 TCP 与 UDP 的连接方式、可靠性、开销和适用场景。",
      segmentIds: ["demo-segment-3"],
      startMs: 17100,
      endMs: 25100,
    },
  ];
  return {
    id: `demo-${Date.now()}`,
    title: fileName.replace(/\.[^.]+$/, ""),
    createdAt: now,
    durationMs: 25100,
    status: "completed",
    modelId,
    providerName: "演示记录",
    lexiconName: null,
    sourcePath: fileName,
    audioPath: null,
    language,
    rawSegments,
    calibratedSegments,
    examPoints,
  };
}

export default function App() {
  const [locale, setLocale] = useState<Locale>(
    () => (localStorage.getItem("verilecture-locale") as Locale) || "zh-CN",
  );
  const t = useMemo(() => createTranslator(locale), [locale]);
  const [snapshot, setSnapshot] = useState<AppSnapshot>(emptySnapshot);
  const [view, setView] = useState<ViewId>("import");
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [tutorialOpen, setTutorialOpen] = useState(false);
  const [selectedRecord, setSelectedRecord] = useState<RecordDetail | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    localStorage.setItem("verilecture-locale", locale);
  }, [locale]);

  useEffect(() => {
    // Toasts hold rendered copy rather than translation keys; never leave a
    // message from the previous locale visible after switching languages.
    setToast(null);
  }, [locale]);

  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        if (isTauri) {
          const data = await getSnapshot();
          if (!alive) return;
          setSnapshot(data);
          setLocale(data.locale || locale);
          setOnboardingOpen(!data.onboardingComplete);
          setTutorialOpen(
            data.onboardingComplete &&
              localStorage.getItem("verilecture-tutorial") !== "done",
          );
        } else {
          const done =
            localStorage.getItem("verilecture-demo-onboarding") === "done";
          setOnboardingOpen(!done);
          setTutorialOpen(
            done && localStorage.getItem("verilecture-tutorial") !== "done",
          );
        }
      } catch {
        if (alive) {
          const done =
            localStorage.getItem("verilecture-demo-onboarding") === "done";
          setOnboardingOpen(!done);
          setTutorialOpen(
            done && localStorage.getItem("verilecture-tutorial") !== "done",
          );
        }
      } finally {
        if (alive) setLoading(false);
      }
    })();
    return () => {
      alive = false;
    };
  }, []);

  useEffect(() => {
    if (!isTauri) return;
    let dispose: (() => void) | undefined;
    subscribeInstallProgress((progress) =>
      setSnapshot((current) => ({
        ...current,
        models: current.models.map((model) =>
          model.id === progress.modelId
            ? {
                ...model,
                status:
                  progress.stage === "ready"
                    ? "ready"
                    : progress.stage === "error"
                      ? "error"
                      : progress.stage === "verifying"
                        ? "verifying"
                        : "downloading",
              }
            : model,
        ),
      })),
    ).then((unlisten) => {
      dispose = unlisten;
    });
    return () => dispose?.();
  }, []);

  const refresh = async () => {
    try {
      const data = await getSnapshot();
      setSnapshot(data);
      setLocale(data.locale || locale);
    } catch {
      /* browser preview has a deterministic demo state */
    }
  };

  const onLocaleChange = (next: Locale) => {
    setLocale(next);
    if (isTauri)
      void call<void>("set_locale", { locale: next }).catch(() =>
        setToast(t("errorGeneric")),
      );
  };

  const openRecord = async (id: string) => {
    try {
      setSelectedRecord(await getRecord(id));
    } catch {
      setSelectedRecord(
        snapshot.records.find(
          (record) => record.id === id,
        ) as RecordDetail | null,
      );
    }
  };

  const handleDelete = async (id: string, deleteCopy: boolean) => {
    try {
      if (isTauri) await deleteRecord(id, deleteCopy);
    } catch {
      /* keep UI responsive */
    }
    setSnapshot((current) => ({
      ...current,
      records: current.records.filter((record) => record.id !== id),
    }));
    if (selectedRecord?.id === id) setSelectedRecord(null);
    setToast(t("sourceNotDeleted"));
  };

  const finishOnboarding = async () => {
    if (isTauri) {
      try {
        await completeOnboarding();
        await refresh();
      } catch (error) {
        const code = error instanceof Error ? error.message : String(error);
        setToast(
          code === "PROVIDER_CONSENT_REQUIRED"
            ? t("providerConsentRequired")
            : t("onboardingIncomplete"),
        );
        return;
      }
    }
    setOnboardingOpen(false);
    localStorage.setItem("verilecture-demo-onboarding", "done");
    setTutorialOpen(true);
  };

  if (loading)
    return (
      <div className="splash">
        <div className="splash-mark">课</div>
        <p>
          {t("brand")} · {t("processing")}
        </p>
      </div>
    );

  return (
    <div className="app-shell">
      <Sidebar
        view={view}
        setView={setView}
        locale={locale}
        setLocale={onLocaleChange}
        t={t}
        hardware={snapshot.hardware}
        model={modelLabel(snapshot.selectedModelId, snapshot.models)}
      />
      <main className="main-area">
        <div className="topbar">
          <div className="mobile-brand">
            <span className="mobile-mark">课</span>
            <span>{t("brand")}</span>
          </div>
          <div className="topbar-status">
            <StatusDot
              ok={snapshot.models.some((model) => model.status === "ready")}
              label={
                snapshot.models.some((model) => model.status === "ready")
                  ? t("localReady")
                  : t("modelMissing")
              }
            />
            <StatusDot
              ok={Boolean(snapshot.provider?.tested)}
              label={
                snapshot.provider?.tested
                  ? t("providerReady")
                  : t("providerMissing")
              }
            />
          </div>
          <button className="icon-button" aria-label={t("menu")}>
            <Menu size={19} />
          </button>
        </div>
        <div className="content-wrap">
          {selectedRecord ? (
            <RecordDetailViewV2
              detail={selectedRecord}
              setDetail={setSelectedRecord}
              locale={locale}
              t={t}
              setToast={setToast}
              onBack={() => setSelectedRecord(null)}
            />
          ) : view === "import" ? (
            <ImportView
              snapshot={snapshot}
              setSnapshot={setSnapshot}
              onOpenRecord={openRecord}
              setToast={setToast}
              locale={locale}
              t={t}
            />
          ) : null}
          {!selectedRecord && view === "records" ? (
            <RecordsView
              records={snapshot.records}
              locale={locale}
              t={t}
              onOpen={openRecord}
              onDelete={handleDelete}
            />
          ) : null}
          {!selectedRecord && view === "lexicons" ? (
            <LexiconsView
              lexicons={snapshot.lexicons}
              t={t}
              setSnapshot={setSnapshot}
              setToast={setToast}
            />
          ) : null}
          {!selectedRecord && view === "settings" ? (
            <SettingsView
              snapshot={snapshot}
              setSnapshot={setSnapshot}
              refresh={refresh}
              locale={locale}
              setLocale={onLocaleChange}
              t={t}
              setToast={setToast}
            />
          ) : null}
          {!selectedRecord && view === "about" ? <AboutView t={t} /> : null}
        </div>
      </main>
      {onboardingOpen ? (
        <OnboardingModal
          snapshot={snapshot}
          setSnapshot={setSnapshot}
          close={finishOnboarding}
          locale={locale}
          setLocale={onLocaleChange}
          t={t}
          setToast={setToast}
        />
      ) : null}
      {tutorialOpen && !onboardingOpen ? (
        <TutorialModal
          close={() => {
            setTutorialOpen(false);
            localStorage.setItem("verilecture-tutorial", "done");
          }}
          t={t}
        />
      ) : null}
      {toast ? (
        <div className="toast" role="status">
          <Check size={16} />
          {toast}
          <button onClick={() => setToast(null)} aria-label={t("close")}>
            <X size={14} />
          </button>
        </div>
      ) : null}
    </div>
  );
}

function StatusDot({ ok, label }: { ok: boolean; label: string }) {
  return (
    <span className={`status-dot ${ok ? "is-ok" : ""}`}>
      <i />
      {label}
    </span>
  );
}

function Sidebar({
  view,
  setView,
  locale,
  setLocale,
  hardware,
  model,
  t,
}: {
  view: ViewId;
  setView: (view: ViewId) => void;
  locale: Locale;
  setLocale: (locale: Locale) => void;
  hardware: HardwareProfile | null;
  model: string;
  t: (key: string) => string;
}) {
  const items: { id: ViewId; label: string; icon: typeof Upload }[] = [
    { id: "import", label: t("navImport"), icon: Upload },
    { id: "records", label: t("navRecords"), icon: Headphones },
    { id: "lexicons", label: t("navLexicons"), icon: Library },
  ];
  return (
    <aside className="sidebar">
      <div className="brand-lockup">
        <div className="brand-symbol">课</div>
        <div>
          <div className="brand-name">{t("brand")}</div>
          <div className="brand-sub">{t("brandSub")}</div>
        </div>
      </div>
      <div className="sidebar-rule" />
      <nav className="main-nav" aria-label={t("mainNavigation")}>
        {items.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            className={`nav-item ${view === id ? "active" : ""}`}
            onClick={() => setView(id)}
          >
            <Icon size={18} strokeWidth={1.8} />
            <span>{label}</span>
            {id === "import" ? <span className="nav-arrow">↗</span> : null}
          </button>
        ))}
      </nav>
      <div className="sidebar-note">
        <div className="note-eyebrow">{t("privacyLocal").split("。")[0]}</div>
        <p>{t("privacyCloud")}</p>
        <ShieldCheck size={18} />
      </div>
      <div className="sidebar-bottom">
        <div className="hardware-mini">
          <div className="mini-icon">
            <Cpu size={15} />
          </div>
          <div>
            <span>
              {hardware?.nvidiaDetected ? hardware.gpuName : t("cpu")}
            </span>
            <small>{model}</small>
          </div>
          <span className="ready-pill">
            <i />
          </span>
        </div>
        <button
          className={`bottom-item ${view === "settings" ? "active" : ""}`}
          onClick={() => setView("settings")}
        >
          <Settings2 size={17} />
          {t("navSettings")}
        </button>
        <button
          className={`bottom-item ${view === "about" ? "active" : ""}`}
          onClick={() => setView("about")}
        >
          <Info size={17} />
          {t("navAbout")}
        </button>
        <div className="language-switch">
          <Languages size={15} />
          <button
            className={locale === "zh-CN" ? "selected" : ""}
            onClick={() => setLocale("zh-CN")}
          >
            中
          </button>
          <span>/</span>
          <button
            className={locale === "en-US" ? "selected" : ""}
            onClick={() => setLocale("en-US")}
          >
            EN
          </button>
        </div>
      </div>
    </aside>
  );
}

function ImportView({
  snapshot,
  setSnapshot,
  onOpenRecord,
  setToast,
  locale,
  t,
}: {
  snapshot: AppSnapshot;
  setSnapshot: Dispatch<SetStateAction<AppSnapshot>>;
  onOpenRecord: (id: string) => void;
  setToast: (message: string) => void;
  locale: Locale;
  t: (key: string) => string;
}) {
  const [file, setFile] = useState<File | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [language, setLanguage] = useState<AudioLanguage>("auto");
  const [lexiconId, setLexiconId] = useState<string>("");
  const [busy, setBusy] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);
  const [jobProgress, setJobProgress] = useState<{
    stage: string;
    progressPercent: number;
    message: string;
  } | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const recent = snapshot.records.slice(0, 3);
  const selectedModel =
    snapshot.models.find((model) => model.id === snapshot.selectedModelId) ??
    snapshot.models.find((model) => model.status === "ready");

  useEffect(() => {
    if (!isTauri || !jobId) return;
    let active = true;
    let dispose: (() => void) | undefined;
    void subscribeAudioProgress((event) => {
      if (active && event.jobId === jobId) setJobProgress(event);
    }).then((unlisten) => {
      if (active) dispose = unlisten;
      else unlisten();
    });
    return () => {
      active = false;
      dispose?.();
    };
  }, [jobId]);

  const chooseAudio = async () => {
    try {
      if (isTauri) {
        const path = await pickFile("audio");
        if (path) {
          const name = path.split(/[\\/]/).pop() || path;
          setSelectedPath(path);
          setFile({ name, size: 0 } as File);
        }
      } else inputRef.current?.click();
    } catch {
      setToast(t("errorGeneric"));
    }
  };

  const handleProcess = async () => {
    if (!file) return;
    if (!selectedModel || selectedModel.status !== "ready") {
      setToast(t("modelNotReady"));
      return;
    }
    setBusy(true);
    const nextJobId =
      typeof crypto !== "undefined" && "randomUUID" in crypto
        ? crypto.randomUUID()
        : `job-${Date.now()}`;
    setJobId(nextJobId);
    setJobProgress({
      stage: "queued",
      progressPercent: 0,
      message: t("processing"),
    });
    try {
      const filePath =
        selectedPath ?? (file as File & { path?: string }).path ?? file.name;
      const result = await importAudio({
        path: filePath,
        title: file.name.replace(/\.[^.]+$/, ""),
        language,
        lexiconId: lexiconId || null,
        jobId: nextJobId,
      });
      setSnapshot((current) => ({
        ...current,
        records: [result, ...current.records],
      }));
      setFile(null);
      setSelectedPath(null);
      setToast(t("statusCompleted"));
      onOpenRecord(result.id);
    } catch {
      if (!isTauri) {
        const result = makeDemoRecord(file.name, language, selectedModel.id);
        setSnapshot((current) => ({
          ...current,
          records: [result, ...current.records],
        }));
        setFile(null);
        setSelectedPath(null);
        onOpenRecord(result.id);
        setToast(t("browserPreview"));
      } else setToast(t("errorGeneric"));
    } finally {
      setBusy(false);
      setJobId(null);
      setJobProgress(null);
    }
  };

  const cancelCurrentJob = async () => {
    if (!jobId || !isTauri) return;
    try {
      await cancelAudioJob(jobId);
      setToast(t("statusCancelled"));
    } catch {
      setToast(t("errorGeneric"));
    }
  };

  return (
    <section className="page page-import">
      <PageIntro
        eyebrow={t("eyebrowAudioIntake")}
        title={t("importTitle")}
        lead={t("importLead")}
      />
      <div className="status-ribbon">
        <div>
          <span className="ribbon-label">
            {selectedModel?.status === "ready"
              ? t("localReady")
              : t("modelMissing")}
          </span>
          <strong>{selectedModel?.name ?? t("modelMissing")}</strong>
        </div>
        <div>
          <span className="ribbon-label">{t("provider")}</span>
          <strong>{snapshot.provider?.modelId ?? t("providerMissing")}</strong>
        </div>
        <div className="ribbon-lock">
          <LockKeyhole size={16} />
          {t("privacyLocal")}
        </div>
      </div>
      <div className="import-grid">
        <div
          className={`drop-zone ${file ? "has-file" : ""}`}
          onClick={() => void chooseAudio()}
          onDragOver={(event) => event.preventDefault()}
          onDrop={(event) => {
            event.preventDefault();
            const dropped = event.dataTransfer.files[0];
            if (dropped) {
              setSelectedPath(
                (dropped as File & { path?: string }).path ?? null,
              );
              setFile(dropped);
            }
          }}
        >
          <input
            ref={inputRef}
            type="file"
            accept="audio/*,video/mp4"
            hidden
            onChange={(event) => {
              const picked = event.target.files?.[0] ?? null;
              setFile(picked);
              setSelectedPath(
                (picked as (File & { path?: string }) | null)?.path ?? null,
              );
            }}
          />
          {file ? (
            <>
              <div className="file-orb">
                <FileAudio size={27} />
              </div>
              <div className="drop-title">{file.name}</div>
              <div className="drop-meta">
                {file.size ? formatBytes(file.size) : t("chooseAudio")}
              </div>
              <button
                className="text-button"
                onClick={(event) => {
                  event.stopPropagation();
                  setFile(null);
                  setSelectedPath(null);
                }}
              >
                {t("cancel")}
              </button>
            </>
          ) : (
            <>
              <div className="drop-graphic">
                <Upload size={28} />
              </div>
              <div className="drop-title">{t("dropAudio")}</div>
              <div className="drop-meta">{t("supportedFormats")}</div>
            </>
          )}
        </div>
        <div className="import-options">
          <div className="option-heading">
            <span>02</span>
            <div>
              <h2>{t("audioLanguage")}</h2>
              <p>{t("audioLanguageLead")}</p>
            </div>
          </div>
          <div className="segmented">
            {(
              [
                ["auto", "languageAuto"],
                ["zh", "languageZh"],
                ["en", "languageEn"],
                ["yue", "languageYue"],
              ] as const
            ).map(([value, key]) => (
              <button
                key={value}
                className={language === value ? "selected" : ""}
                onClick={() => setLanguage(value)}
              >
                {t(key)}
              </button>
            ))}
          </div>
          <div className="option-heading second">
            <span>03</span>
            <div>
              <h2>{t("lexiconOptional")}</h2>
              <p>{t("lexiconLeadShort")}</p>
            </div>
          </div>
          <select
            className="field-select"
            value={lexiconId}
            onChange={(event) => setLexiconId(event.target.value)}
          >
            <option value="">{t("noLexicon")}</option>
            {snapshot.lexicons.map((lexicon) => (
              <option value={lexicon.id} key={lexicon.id}>
                {lexicon.name}
              </option>
            ))}
          </select>
          <button
            className="primary-button process-button"
            disabled={!file || busy}
            onClick={handleProcess}
          >
            {busy ? (
              <>
                <RefreshCw className="spin" size={17} />
                {t("processing")}
              </>
            ) : (
              <>
                <Zap size={17} />
                {t("startProcessing")}
              </>
            )}
          </button>
          {busy && jobProgress ? (
            <div className="audio-job-progress" aria-live="polite">
              <div>
                <span>{audioProgressMessage(jobProgress.message, t)}</span>
                <strong>{jobProgress.progressPercent}%</strong>
              </div>
              <div className="progress-track">
                <i style={{ width: `${jobProgress.progressPercent}%` }} />
              </div>
              {isTauri ? (
                <button
                  className="text-button"
                  onClick={() => void cancelCurrentJob()}
                >
                  {t("cancel")}
                </button>
              ) : null}
            </div>
          ) : null}
        </div>
      </div>
      <div className="section-heading">
        <div>
          <span className="eyebrow">{t("eyebrowTraceHistory")}</span>
          <h2>{t("recentRecords")}</h2>
        </div>
        <button className="quiet-button" disabled={!recent.length}>
          {t("navRecords")} <ArrowRight size={16} />
        </button>
      </div>
      {recent.length ? (
        <div className="record-strip">
          {recent.map((record) => (
            <MiniRecord
              key={record.id}
              record={record}
              locale={locale}
              t={t}
              onOpen={() => onOpenRecord(record.id)}
            />
          ))}
        </div>
      ) : (
        <EmptyState
          icon={<FileText size={24} />}
          title={t("emptyRecords")}
          lead={t("emptyRecordsLead")}
        />
      )}
    </section>
  );
}

function PageIntro({
  eyebrow,
  title,
  lead,
}: {
  eyebrow: string;
  title: string;
  lead: string;
}) {
  return (
    <div className="page-intro">
      <span className="eyebrow">{eyebrow}</span>
      <h1>{title}</h1>
      <p>{lead}</p>
      <div className="intro-mark">○</div>
    </div>
  );
}

function MiniRecord({
  record,
  locale,
  t,
  onOpen,
}: {
  record: RecordSummary;
  locale: Locale;
  t: (key: string) => string;
  onOpen: () => void;
}) {
  return (
    <button className="mini-record" onClick={onOpen}>
      <div className="mini-record-icon">
        <Headphones size={17} />
      </div>
      <div className="mini-record-copy">
        <strong>{record.title}</strong>
        <span>
          {formatDate(record.createdAt, locale)} ·{" "}
          {formatDuration(record.durationMs)}
        </span>
      </div>
      <span className="record-status">
        {record.status === "completed"
          ? t("statusCompleted")
          : t(
              `status${record.status[0].toUpperCase()}${record.status.slice(1)}`,
            )}
      </span>
      <ArrowRight size={16} />
    </button>
  );
}

function RecordsView({
  records,
  locale,
  t,
  onOpen,
  onDelete,
}: {
  records: RecordSummary[];
  locale: Locale;
  t: (key: string) => string;
  onOpen: (id: string) => void;
  onDelete: (id: string, deleteCopy: boolean) => void;
}) {
  const [query, setQuery] = useState("");
  const filtered = records.filter((record) =>
    record.title.toLocaleLowerCase().includes(query.toLocaleLowerCase()),
  );
  return (
    <section className="page">
      <PageIntro
        eyebrow={t("eyebrowAudioArchive")}
        title={t("recordsTitle")}
        lead={t("recordsLead")}
      />
      <div className="toolbar">
        <div className="search-field">
          <Search size={17} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("search")}
          />
        </div>
        <button className="quiet-button">
          <MoreHorizontal size={17} />
          {records.length}
        </button>
      </div>
      {filtered.length ? (
        <div className="records-table">
          {filtered.map((record) => (
            <div className="record-row" key={record.id}>
              <div className="row-main">
                <div className="row-icon">
                  <Headphones size={18} />
                </div>
                <div>
                  <strong>{record.title}</strong>
                  <span>
                    {formatDate(record.createdAt, locale)} ·{" "}
                    {formatDuration(record.durationMs)}
                  </span>
                </div>
              </div>
              <div className="row-fact">{modelShort(record.modelId)}</div>
              <div className="row-fact">
                {record.lexiconName ?? t("noLexicon")}
              </div>
              <span className={`status-badge ${record.status}`}>
                {record.status === "completed"
                  ? t("statusCompleted")
                  : record.status === "processing"
                    ? t("statusProcessing")
                    : t("statusFailed")}
              </span>
              <button className="row-open" onClick={() => onOpen(record.id)}>
                {t("openRecord")} <ArrowRight size={15} />
              </button>
              <button
                className="row-delete"
                title={t("deleteRecord")}
                onClick={() => onDelete(record.id, false)}
              >
                <X size={15} />
              </button>
            </div>
          ))}
        </div>
      ) : (
        <EmptyState
          icon={<Headphones size={24} />}
          title={t("emptyRecords")}
          lead={t("emptyRecordsLead")}
        />
      )}
    </section>
  );
}

function modelShort(id: ModelId) {
  return id === "qwen3-asr-1.7b"
    ? "Qwen 1.7B"
    : id === "qwen3-asr-0.6b"
      ? "Qwen 0.6B"
      : "Fun Nano";
}
function providerProtocol(provider: string): ProviderConfig["protocol"] {
  return provider === "OpenAI"
    ? "openai_responses"
    : provider === "Anthropic"
      ? "anthropic_messages"
      : provider === "Google Gemini"
        ? "gemini_generate_content"
        : "openai_compatible";
}
function modelDescription(id: ModelId, t: (key: string) => string) {
  return id === "qwen3-asr-1.7b"
    ? t("modelQwen17Description")
    : id === "qwen3-asr-0.6b"
      ? t("modelQwen06Description")
      : t("modelFunDescription");
}
function modelReason(model: ModelOption, t: (key: string) => string) {
  if (model.reason.includes("等待")) return t("hardwareScanRequired");
  if (model.reason.includes("will be downloaded")) {
    return t("cudaRuntimeDownloadReason");
  }
  if (model.reason.includes("CUDA")) return t("cudaUnavailableReason");
  if (model.id === "fun-asr-nano-2512" && model.reason.includes("内存")) {
    return t("cpuRamUnavailableReason");
  }
  if (model.reason.includes("内存")) return t("ramUnavailableReason");
  if (model.reason.includes("显存")) return t("vramUnavailableReason");
  if (model.reason.includes("磁盘")) return t("diskUnavailableReason");
  if (model.reason.includes("目录")) return t("directoryUnavailableReason");
  return model.reason;
}

function RecordDetailViewV2({
  detail,
  setDetail,
  locale,
  t,
  setToast,
  onBack,
}: {
  detail: RecordDetail;
  setDetail: Dispatch<SetStateAction<RecordDetail | null>>;
  locale: Locale;
  t: (key: string) => string;
  setToast: (message: string) => void;
  onBack: () => void;
}) {
  const [tab, setTab] = useState<"points" | "transcript">("points");
  const [source, setSource] = useState<"raw" | "calibrated">("calibrated");
  const [playingMs, setPlayingMs] = useState<number | null>(null);
  const [currentMs, setCurrentMs] = useState(0);
  const [analysisBusy, setAnalysisBusy] = useState(false);
  const audioRef = useRef<HTMLAudioElement>(null);
  const audioUrl = audioAssetUrl(detail.audioPath);
  const segments =
    source === "raw" ? detail.rawSegments : detail.calibratedSegments;

  const playAt = (startMs: number) => {
    setCurrentMs(startMs);
    setPlayingMs(startMs);
    if (audioRef.current) {
      audioRef.current.currentTime = startMs / 1000;
      void audioRef.current
        .play()
        .catch(() => setToast(t("audioPlaybackFailed")));
    }
  };
  const togglePlay = () => {
    if (!audioRef.current) {
      setPlayingMs(playingMs === null ? currentMs : null);
      return;
    }
    if (playingMs === null) {
      if (audioRef.current.currentTime >= audioRef.current.duration)
        audioRef.current.currentTime = 0;
      void audioRef.current
        .play()
        .then(() => setPlayingMs((audioRef.current?.currentTime ?? 0) * 1000))
        .catch(() => setToast(t("audioPlaybackFailed")));
    } else {
      audioRef.current.pause();
      setPlayingMs(null);
    }
  };
  const runAnalysis = async () => {
    if (!isTauri) {
      setToast(t("browserPreview"));
      return;
    }
    setAnalysisBusy(true);
    try {
      const result = await generateExamPoints(detail.id);
      setDetail(result);
      setToast(t("analysisCompleted"));
    } catch (error) {
      setToast(analysisErrorMessage(error, t));
    } finally {
      setAnalysisBusy(false);
    }
  };
  const runExport = async (format: "json" | "md" | "txt") => {
    if (!isTauri) {
      setToast(t("browserPreview"));
      return;
    }
    try {
      const path = await pickSaveFile(format);
      if (path) {
        await exportRecord(detail.id, path, format);
        setToast(t("exportCompleted"));
      }
    } catch {
      setToast(t("exportFailed"));
    }
  };
  return (
    <section className="page detail-page">
      <button className="back-link" onClick={onBack}>
        <ArrowLeft size={17} />
        {t("navRecords")}
      </button>
      <div className="detail-head">
        <div>
          <span className="eyebrow">
            {t("audioRecord")} / {formatDate(detail.createdAt, locale)}
          </span>
          <h1>{detail.title}</h1>
          <div className="detail-meta">
            <span>
              <Headphones size={15} />
              {formatDuration(detail.durationMs)}
            </span>
            <span>
              <Zap size={15} />
              {modelShort(detail.modelId)}
            </span>
            <span>
              <BookOpen size={15} />
              {detail.lexiconName ?? t("noLexicon")}
            </span>
          </div>
        </div>
        <div className="player-card">
          <div className="player-progress">
            <span
              style={{
                width: `${Math.min(100, detail.durationMs ? (currentMs / detail.durationMs) * 100 : 0)}%`,
              }}
            />
          </div>
          <div className="player-controls">
            <button className="round-button" onClick={togglePlay}>
              {playingMs === null ? (
                <Play size={17} fill="currentColor" />
              ) : (
                <Square size={14} fill="currentColor" />
              )}
            </button>
            <span>
              {formatDuration(currentMs)} / {formatDuration(detail.durationMs)}
            </span>
            <Headphones size={16} />
          </div>
          <audio
            ref={audioRef}
            src={audioUrl ?? undefined}
            preload="metadata"
            onTimeUpdate={(event) => {
              const value = event.currentTarget.currentTime * 1000;
              setCurrentMs(value);
              setPlayingMs(value);
            }}
            onPlay={() => setPlayingMs(currentMs)}
            onPause={() => setPlayingMs(null)}
            onEnded={() => {
              setPlayingMs(null);
              setCurrentMs(detail.durationMs);
            }}
          />
        </div>
      </div>
      <div className="detail-actions">
        <button
          className="primary-button"
          onClick={runAnalysis}
          disabled={analysisBusy}
        >
          {analysisBusy ? (
            <>
              <RefreshCw className="spin" size={16} />
              {t("processing")}
            </>
          ) : (
            <>
              <Sparkles size={16} />
              {detail.examPoints.length
                ? t("regeneratePoints")
                : t("generatePoints")}
            </>
          )}
        </button>
        <button className="quiet-button" onClick={() => void runExport("md")}>
          {t("exportMarkdown")}
        </button>
        <button className="quiet-button" onClick={() => void runExport("json")}>
          {t("exportJson")}
        </button>
      </div>
      <div className="ai-notice">
        <Sparkles size={16} />
        {t("aiNotice")}
      </div>
      <div className="detail-tabs">
        <button
          className={tab === "points" ? "selected" : ""}
          onClick={() => setTab("points")}
        >
          {t("examPoints")} <span>{detail.examPoints.length}</span>
        </button>
        <button
          className={tab === "transcript" ? "selected" : ""}
          onClick={() => setTab("transcript")}
        >
          {t("transcript")} <span>{segments.length}</span>
        </button>
      </div>
      {tab === "points" ? (
        <ExamPoints points={detail.examPoints} t={t} onListen={playAt} />
      ) : (
        <TranscriptView
          segments={segments}
          source={source}
          setSource={setSource}
          t={t}
          onListen={playAt}
        />
      )}
    </section>
  );
}

function ExamPoints({
  points,
  t,
  onListen,
}: {
  points: ExamPoint[];
  t: (key: string) => string;
  onListen: (ms: number) => void;
}) {
  if (!points.length)
    return (
      <EmptyState
        icon={<Sparkles size={24} />}
        title={t("noExamPoints")}
        lead=""
      />
    );
  const chapters = [...new Set(points.map((point) => point.chapterTitle))];
  const chapterLabel = (chapter: string) =>
    chapter === "UNMATCHED" ? t("unmatchedChapter") : chapter;
  return (
    <div className="points-list">
      {chapters.map((chapter) => (
        <div className="chapter-group" key={chapter}>
          <div className="chapter-label">
            <BookOpen size={16} />
            {chapterLabel(chapter)}
          </div>
          {points
            .filter((point) => point.chapterTitle === chapter)
            .map((point, index) => (
              <article className="point-card" key={point.id}>
                <span className="point-number">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <div className="point-copy">
                  <h3>{point.title}</h3>
                  <p>{point.detail}</p>
                  <span className="evidence-line">
                    {point.segmentIds.length} {t("transcript")} ·{" "}
                    {formatDuration(point.startMs)}
                  </span>
                </div>
                <button
                  className="listen-button"
                  onClick={() => onListen(point.startMs)}
                >
                  <Headphones size={15} />
                  {t("listen")}
                </button>
              </article>
            ))}
        </div>
      ))}
    </div>
  );
}

function TranscriptView({
  segments,
  source,
  setSource,
  t,
  onListen,
}: {
  segments: TranscriptSegment[];
  source: "raw" | "calibrated";
  setSource: (source: "raw" | "calibrated") => void;
  t: (key: string) => string;
  onListen: (ms: number) => void;
}) {
  return (
    <div className="transcript-wrap">
      <div className="transcript-toggle">
        <button
          className={source === "calibrated" ? "selected" : ""}
          onClick={() => setSource("calibrated")}
        >
          {t("calibratedTranscript")}
        </button>
        <button
          className={source === "raw" ? "selected" : ""}
          onClick={() => setSource("raw")}
        >
          {t("rawTranscript")}
        </button>
      </div>
      <div className="transcript-list">
        {segments.map((segment) => (
          <button
            className="transcript-line"
            key={segment.id}
            onClick={() => onListen(segment.startMs)}
          >
            <span className="timestamp">{formatDuration(segment.startMs)}</span>
            <span>{segment.text}</span>
            <Headphones size={15} />
          </button>
        ))}
      </div>
      <div className="raw-safe-note">
        <ShieldCheck size={15} />
        {t("rawTranscript")} {t("sourceNotDeleted")}
      </div>
    </div>
  );
}

function LexiconsView({
  lexicons,
  t,
  setSnapshot,
  setToast,
}: {
  lexicons: LexiconSummary[];
  t: (key: string) => string;
  setSnapshot: Dispatch<SetStateAction<AppSnapshot>>;
  setToast: (message: string) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [busy, setBusy] = useState(false);
  const [excerptConsent, setExcerptConsent] = useState(false);
  const [structuredConsent, setStructuredConsent] = useState(false);
  const [generatingId, setGeneratingId] = useState<string | null>(null);
  const [editing, setEditing] = useState<LexiconProfile | null>(null);
  const [saving, setSaving] = useState(false);
  const importBook = async (path: string, name: string) => {
    setBusy(true);
    try {
      if (!isTauri) {
        const preview: LexiconSummary = {
          id: `preview-lexicon-${Date.now()}`,
          name,
          textbookTitle: name,
          version: 1,
          terminologyCount: 3,
          chapterCount: 2,
          updatedAt: new Date().toISOString(),
          status: "ready",
        };
        setSnapshot((current) => ({
          ...current,
          lexicons: [preview, ...current.lexicons],
        }));
        setToast(t("browserPreview"));
      } else {
        const result = await importLexicon(path, name);
        setSnapshot((current) => ({
          ...current,
          lexicons: [result, ...current.lexicons],
        }));
        setToast(t("lexiconImported"));
      }
    } catch {
      setToast(t("errorGeneric"));
    } finally {
      setBusy(false);
    }
  };
  const updateConsent = async (
    type: "cloud_llm_textbook_excerpt" | "cloud_llm_lexicon_structured_data",
    granted: boolean,
  ) => {
    if (type === "cloud_llm_textbook_excerpt") setExcerptConsent(granted);
    else setStructuredConsent(granted);
    if (!isTauri) return;
    try {
      await setPrivacyConsent(type, granted);
    } catch {
      if (type === "cloud_llm_textbook_excerpt") setExcerptConsent(!granted);
      else setStructuredConsent(!granted);
      setToast(t("errorGeneric"));
    }
  };
  const generate = async (lexicon: LexiconSummary) => {
    if (!excerptConsent || !structuredConsent) {
      setToast(t("dataConsentRequired"));
      return;
    }
    setGeneratingId(lexicon.id);
    setToast(t("lexiconGenerationStarted"));
    try {
      if (isTauri) {
        const updated = await generateLexicon(lexicon.id);
        setSnapshot((current) => ({
          ...current,
          lexicons: current.lexicons.map((item) =>
            item.id === updated.id ? updated : item,
          ),
        }));
      }
      setToast(isTauri ? t("lexiconGenerationCompleted") : t("browserPreview"));
    } catch {
      setToast(t("errorGeneric"));
    } finally {
      setGeneratingId(null);
    }
  };
  const chooseBook = async () => {
    try {
      if (isTauri) {
        const path = await pickFile("textbook");
        if (path)
          await importBook(path, path.split(/[\\/]/).pop() || t("newLexicon"));
      } else inputRef.current?.click();
    } catch {
      setToast(t("errorGeneric"));
    }
  };
  const openEditor = async (lexicon: LexiconSummary) => {
    try {
      if (isTauri) {
        setEditing(await getLexicon(lexicon.id));
      } else {
        const sourceDocumentId = `preview-document-${lexicon.id}`;
        setEditing({
          id: lexicon.id,
          name: lexicon.name,
          version: lexicon.version,
          textbookTitle: lexicon.textbookTitle,
          sourceDocumentId,
          chapters: [],
          terms: ["ASR", "VAD", "Forced alignment"].map(
            (canonicalTerm, index) => ({
              id: `${lexicon.id}-term-${index + 1}`,
              canonicalTerm,
              aliases: [],
              abbreviation: null,
              englishName: null,
              definition: null,
              chapterIds: [],
              commonAsrErrors: [],
              sourceReferences: [sourceDocumentId],
              confirmedByUser: false,
            }),
          ),
          correctionRules: [],
          createdAt: lexicon.updatedAt,
          updatedAt: lexicon.updatedAt,
        });
      }
    } catch {
      setToast(t("errorGeneric"));
    }
  };
  const updateTerm = (index: number, patch: Partial<LexiconTerm>) => {
    setEditing((current) =>
      current
        ? {
            ...current,
            terms: current.terms.map((term, termIndex) =>
              termIndex === index ? { ...term, ...patch } : term,
            ),
          }
        : current,
    );
  };
  const save = async () => {
    if (!editing) return;
    const terms = editing.terms.filter((term) => term.canonicalTerm.trim());
    setSaving(true);
    try {
      if (isTauri) {
        const summary = await saveLexicon({ ...editing, terms });
        setSnapshot((current) => ({
          ...current,
          lexicons: current.lexicons.map((item) =>
            item.id === summary.id ? summary : item,
          ),
        }));
      } else {
        setSnapshot((current) => ({
          ...current,
          lexicons: current.lexicons.map((item) =>
            item.id === editing.id
              ? {
                  ...item,
                  name: editing.name,
                  textbookTitle: editing.textbookTitle,
                  version: item.version + 1,
                  terminologyCount: terms.length,
                  updatedAt: new Date().toISOString(),
                }
              : item,
          ),
        }));
      }
      setEditing(null);
      setToast(t("lexiconSaved"));
    } catch {
      setToast(t("errorGeneric"));
    } finally {
      setSaving(false);
    }
  };
  return (
    <section className="page">
      <PageIntro
        eyebrow={t("eyebrowLocalKnowledge")}
        title={t("lexiconsTitle")}
        lead={t("lexiconsLead")}
      />
      <div className="section-heading">
        <div>
          <span className="eyebrow">{t("eyebrowTextbookEvidence")}</span>
          <h2>
            {lexicons.length
              ? `${lexicons.length} ${t("lexiconsTitle")}`
              : t("emptyLexicons")}
          </h2>
        </div>
        <>
          <input
            ref={inputRef}
            type="file"
            accept=".pdf,.docx,.pptx,.txt,.md,.markdown"
            hidden
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file)
                void importBook(
                  (file as File & { path?: string }).path || file.name,
                  file.name,
                );
            }}
          />
          <button
            className="primary-button"
            onClick={() => void chooseBook()}
            disabled={busy}
          >
            <BookOpen size={17} />
            {busy ? t("processing") : t("newLexicon")}
          </button>
        </>
      </div>
      <div
        className="lexicon-consent"
        role="group"
        aria-label={t("privacySection")}
      >
        <label className="check-label">
          <input
            type="checkbox"
            checked={excerptConsent}
            onChange={(event) =>
              void updateConsent(
                "cloud_llm_textbook_excerpt",
                event.target.checked,
              )
            }
          />
          {t("lexiconCloudConsent")}
        </label>
        <label className="check-label">
          <input
            type="checkbox"
            checked={structuredConsent}
            onChange={(event) =>
              void updateConsent(
                "cloud_llm_lexicon_structured_data",
                event.target.checked,
              )
            }
          />
          {t("lexiconStructuredConsent")}
        </label>
      </div>
      {lexicons.length ? (
        <div className="lexicon-grid">
          {lexicons.map((lexicon) => (
            <div
              className="lexicon-card"
              key={lexicon.id}
              role="button"
              tabIndex={0}
              onClick={() => void openEditor(lexicon)}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  void openEditor(lexicon);
                }
              }}
            >
              <div className="lexicon-cover">
                <BookOpen size={28} />
                <span>{String(lexicon.version).padStart(2, "0")}</span>
              </div>
              <div className="lexicon-copy">
                <span className="eyebrow">
                  {lexiconStatusLabel(lexicon.status, t)}
                </span>
                <h3>{lexicon.name}</h3>
                <p>{lexicon.textbookTitle}</p>
                <div className="lexicon-facts">
                  <span>
                    {lexicon.terminologyCount} {t("terminology")}
                  </span>
                  <span>
                    {lexicon.chapterCount} {t("chapters")}
                  </span>
                </div>
                <button
                  className="quiet-button lexicon-edit"
                  onClick={(event) => {
                    event.stopPropagation();
                    void openEditor(lexicon);
                  }}
                >
                  <Pencil size={15} />
                  {t("editLexicon")}
                </button>
                <button
                  className="quiet-button lexicon-generate"
                  onClick={(event) => {
                    event.stopPropagation();
                    void generate(lexicon);
                  }}
                  disabled={generatingId === lexicon.id}
                >
                  <Sparkles size={15} />
                  {generatingId === lexicon.id
                    ? t("processing")
                    : t("generateLexicon")}
                </button>
              </div>
              <button
                className="row-delete"
                title={t("deleteLexicon")}
                onClick={async (event) => {
                  event.stopPropagation();
                  try {
                    if (isTauri) await deleteLexicon(lexicon.id);
                    setSnapshot((current) => ({
                      ...current,
                      lexicons: current.lexicons.filter(
                        (item) => item.id !== lexicon.id,
                      ),
                    }));
                  } catch {
                    setToast(t("errorGeneric"));
                  }
                }}
              >
                <X size={15} />
              </button>
              <ArrowRight size={18} />
            </div>
          ))}
        </div>
      ) : (
        <EmptyState
          icon={<Library size={24} />}
          title={t("emptyLexicons")}
          lead={t("emptyLexiconsLead")}
        />
      )}
      {editing ? (
        <div className="modal-backdrop" role="presentation">
          <div
            className="lexicon-editor"
            role="dialog"
            aria-modal="true"
            aria-labelledby="lexicon-editor-title"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="modal-heading">
              <div>
                <span className="eyebrow">{t("lexiconTerms")}</span>
                <h2 id="lexicon-editor-title">{t("editLexicon")}</h2>
              </div>
              <button
                className="icon-button"
                aria-label={t("cancel")}
                onClick={() => setEditing(null)}
              >
                <X size={18} />
              </button>
            </div>
            <p className="editor-lead">{t("lexiconEditorLead")}</p>
            <div className="editor-grid">
              <label className="field-label">
                {t("lexiconName")}
                <input
                  value={editing.name}
                  onChange={(event) =>
                    setEditing({ ...editing, name: event.target.value })
                  }
                />
              </label>
              <label className="field-label">
                {t("textbookTitle")}
                <input
                  value={editing.textbookTitle}
                  onChange={(event) =>
                    setEditing({
                      ...editing,
                      textbookTitle: event.target.value,
                    })
                  }
                />
              </label>
            </div>
            <div className="editor-section-heading">
              <div>
                <span className="eyebrow">
                  {t("versionLabel")} {editing.version}
                </span>
                <h3>{t("lexiconTerms")}</h3>
              </div>
              <button
                className="quiet-button"
                onClick={() =>
                  setEditing({
                    ...editing,
                    terms: [
                      ...editing.terms,
                      {
                        id: `term-${Date.now()}`,
                        canonicalTerm: "",
                        aliases: [],
                        abbreviation: null,
                        englishName: null,
                        definition: null,
                        chapterIds: [],
                        commonAsrErrors: [],
                        sourceReferences: [],
                        confirmedByUser: true,
                      },
                    ],
                  })
                }
              >
                <Plus size={15} />
                {t("addTerm")}
              </button>
            </div>
            <div className="term-editor-list">
              {editing.terms.length ? (
                editing.terms.map((term, index) => (
                  <div className="term-editor-row" key={term.id}>
                    <label className="field-label">
                      {t("canonicalTerm")}
                      <input
                        value={term.canonicalTerm}
                        onChange={(event) =>
                          updateTerm(index, {
                            canonicalTerm: event.target.value,
                          })
                        }
                      />
                    </label>
                    <label className="field-label">
                      {t("definition")}
                      <textarea
                        value={term.definition ?? ""}
                        onChange={(event) =>
                          updateTerm(index, {
                            definition: event.target.value || null,
                          })
                        }
                      />
                    </label>
                    <label className="field-label">
                      {t("aliases")}
                      <input
                        value={term.aliases.join(", ")}
                        onChange={(event) =>
                          updateTerm(index, {
                            aliases: event.target.value
                              .split(",")
                              .map((value) => value.trim())
                              .filter(Boolean),
                          })
                        }
                      />
                    </label>
                    <label className="field-label">
                      {t("asrErrors")}
                      <input
                        value={term.commonAsrErrors.join(", ")}
                        onChange={(event) =>
                          updateTerm(index, {
                            commonAsrErrors: event.target.value
                              .split(",")
                              .map((value) => value.trim())
                              .filter(Boolean),
                          })
                        }
                      />
                    </label>
                    <button
                      className="row-delete term-delete"
                      title={t("removeTerm")}
                      onClick={() =>
                        setEditing({
                          ...editing,
                          terms: editing.terms.filter(
                            (_, termIndex) => termIndex !== index,
                          ),
                        })
                      }
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                ))
              ) : (
                <p className="empty-inline">{t("noTerms")}</p>
              )}
            </div>
            <div className="modal-actions">
              <button className="quiet-button" onClick={() => setEditing(null)}>
                {t("cancel")}
              </button>
              <button
                className="primary-button"
                onClick={() => void save()}
                disabled={saving}
              >
                <Save size={16} />
                {saving ? t("processing") : t("saveLexicon")}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  );
}

function SettingsView({
  snapshot,
  setSnapshot,
  refresh,
  locale,
  setLocale,
  t,
  setToast,
}: {
  snapshot: AppSnapshot;
  setSnapshot: Dispatch<SetStateAction<AppSnapshot>>;
  refresh: () => Promise<void>;
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
  setToast: (message: string) => void;
}) {
  const current =
    snapshot.models.find((model) => model.id === snapshot.selectedModelId) ??
    snapshot.models.find((model) => model.status === "ready");
  const [scanning, setScanning] = useState(false);
  return (
    <section className="page settings-page">
      <PageIntro
        eyebrow={t("eyebrowSettings")}
        title={t("settingsTitle")}
        lead={t("settingsLead")}
      />
      <div className="settings-layout">
        <div className="settings-main">
          <SettingSection
            icon={<MonitorCog size={19} />}
            title={t("hardwareSection")}
            action={
              <button
                className="quiet-button"
                disabled={scanning}
                onClick={async () => {
                  if (scanning) return;
                  setScanning(true);
                  try {
                    const hardware = await scanHardware();
                    const models = await getModelCatalog();
                    setSnapshot((state) => ({ ...state, hardware, models }));
                  } catch {
                    setToast(t("errorGeneric"));
                  } finally {
                    setScanning(false);
                  }
                }}
              >
                <RefreshCw size={15} className={scanning ? "spin" : undefined} />
                {scanning ? t("scanning") : t("scanAgain")}
              </button>
            }
          >
            <div className="hardware-grid">
              <Fact label="CPU" value={snapshot.hardware?.cpuName ?? "—"} />
              <Fact
                label="RAM"
                value={
                  snapshot.hardware
                    ? `${formatBytes(snapshot.hardware.totalRamBytes)} · ${snapshot.hardware.logicalCores} ${t("cores")}`
                    : "—"
                }
              />
              <Fact
                label="GPU"
                value={snapshot.hardware?.gpuName ?? t("cpu")}
              />
              <Fact
                label="CUDA"
                value={
                  snapshot.hardware?.cudaSmokeTest
                    ? t("cudaReady")
                    : t("cudaUnavailable")
                }
              />
            </div>
            <div className="model-current">
              <div className="model-current-icon">
                <Zap size={19} />
              </div>
              <div>
                <span className="eyebrow">{t("localReady")}</span>
                <strong>{current?.name ?? t("modelMissing")}</strong>
                <small>
                  {current?.runtime ?? "—"} ·{" "}
                  {current?.requiresAligner
                    ? t("forcedAlignerTimestamp")
                    : t("nativeTimestamp")}
                </small>
              </div>
              <button
                className="quiet-button"
                onClick={async () => {
                  if (!current) return;
                  try {
                    if (current.status !== "ready")
                      await installModel(current.id);
                    else if (isTauri) await verifyModel(current.id);
                    await refresh();
                    setToast(t("modelStatusReady"));
                  } catch {
                    setToast(t("installFailed"));
                  }
                }}
              >
                {t("repairModel")}
              </button>
            </div>
            <div className="path-line">
              <FolderOpen size={15} />
              {t("modelDirectory")} <code>{snapshot.modelDirectory}</code>
            </div>
          </SettingSection>
          <SettingSection
            icon={<Zap size={19} />}
            title={t("textModelSection")}
            action={
              <span
                className={`small-status ${snapshot.provider?.tested ? "ok" : ""}`}
              >
                <i />
                {snapshot.provider?.tested
                  ? t("connectionPassed")
                  : t("providerMissing")}
              </span>
            }
          >
            <ProviderForm
              initial={snapshot.provider}
              t={t}
              setToast={setToast}
              setSnapshot={setSnapshot}
            />
          </SettingSection>
        </div>
        <aside className="settings-aside">
          <div className="aside-card">
            <ShieldCheck size={21} />
            <h3>{t("privacySection")}</h3>
            <p>{t("privacyLocal")}</p>
            <p>{t("privacyCloud")}</p>
            <a href="#privacy">{t("privacyBoundary")}</a>
          </div>
          <div className="aside-card">
            <Languages size={21} />
            <h3>{t("interfaceSection")}</h3>
            <div className="language-choice">
              <button
                className={locale === "zh-CN" ? "selected" : ""}
                onClick={() => setLocale("zh-CN")}
              >
                {t("chinese")}
              </button>
              <button
                className={locale === "en-US" ? "selected" : ""}
                onClick={() => setLocale("en-US")}
              >
                {t("english")}
              </button>
            </div>
          </div>
        </aside>
      </div>
    </section>
  );
}

function SettingSection({
  icon,
  title,
  action,
  children,
}: {
  icon: ReactNode;
  title: string;
  action?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="setting-section">
      <div className="setting-section-head">
        <div className="setting-title">
          <span>{icon}</span>
          <h2>{title}</h2>
        </div>
        {action}
      </div>
      {children}
    </section>
  );
}
function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div className="fact">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function ProviderForm({
  initial,
  t,
  setToast,
  setSnapshot,
}: {
  initial: ProviderConfig | null;
  t: (key: string) => string;
  setToast: (message: string) => void;
  setSnapshot: Dispatch<SetStateAction<AppSnapshot>>;
}) {
  const [provider, setProvider] = useState(initial?.provider ?? "OpenAI");
  const [baseUrl, setBaseUrl] = useState(
    initial?.baseUrl ?? "https://api.openai.com/v1",
  );
  const [modelId, setModelId] = useState(initial?.modelId ?? "gpt-4o-mini");
  const [apiKey, setApiKey] = useState("");
  const [consent, setConsent] = useState(initial?.consentGranted ?? false);
  const [testing, setTesting] = useState(false);
  const [passed, setPassed] = useState(Boolean(initial?.tested));
  const check = async () => {
    if (!consent) {
      setToast(t("dataConsentRequired"));
      return;
    }
    setTesting(true);
    try {
      const config: ProviderConfig = {
        provider,
        protocol: providerProtocol(provider),
        baseUrl,
        modelId,
        timeoutSeconds: 60,
        configured: true,
        tested: false,
        consentGranted: true,
      };
      const result = await testProvider(config, apiKey);
      setPassed(result.ok);
      if (result.ok) {
        await saveProvider({ ...config, tested: true }, apiKey);
        setSnapshot((snapshot) => ({
          ...snapshot,
          provider: { ...config, tested: true },
        }));
        setToast(t("connectionPassed"));
      } else setToast(result.message || t("connectionFailed"));
    } catch {
      if (!isTauri && apiKey.trim()) {
        setPassed(true);
        setSnapshot((snapshot) => ({
          ...snapshot,
          provider: {
            provider,
            protocol: providerProtocol(provider),
            baseUrl,
            modelId,
            timeoutSeconds: 60,
            configured: true,
            tested: true,
            consentGranted: true,
          },
        }));
        setToast(t("browserPreview"));
      } else {
        setPassed(false);
        setToast(t("connectionFailed"));
      }
    } finally {
      setTesting(false);
    }
  };
  return (
    <div className="provider-form">
      <div className="form-grid">
        <label>
          {t("provider")}
          <select
            className="field-select"
            value={provider}
            onChange={(event) => {
              const value = event.target.value;
              setProvider(value);
              const preset = providerBaseUrl(value);
              if (preset) setBaseUrl(preset);
            }}
          >
            {providerOptions.map(([value, key]) => (
              <option value={value} key={value}>
                {t(key)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("modelId")}
          <input
            className="text-field"
            value={modelId}
            onChange={(event) => setModelId(event.target.value)}
            placeholder={t("modelIdPlaceholder")}
          />
        </label>
        <label className="wide">
          {t("baseUrl")}
          <input
            className="text-field"
            value={baseUrl}
            onChange={(event) => setBaseUrl(event.target.value)}
          />
        </label>
        <label className="wide">
          {t("apiKey")}
          <input
            className="text-field"
            type="password"
            value={apiKey}
            onChange={(event) => setApiKey(event.target.value)}
            placeholder={
              initial?.tested ? t("savedKeyPlaceholder") : t("apiKeyPlaceholder")
            }
          />
        </label>
      </div>
      <div className="provider-footer">
        <label className="check-label">
          <input
            type="checkbox"
            checked={consent}
            onChange={(event) => {
              const granted = event.target.checked;
              setConsent(granted);
              if (isTauri)
                void setPrivacyConsent("cloud_llm_transcript", granted).catch(
                  () => {
                    setConsent(!granted);
                    setToast(t("errorGeneric"));
                  },
                );
            }}
          />
          {t("dataConsent")}
        </label>
        <button
          className="primary-button"
          onClick={check}
          disabled={testing || !baseUrl || !modelId}
        >
          {testing ? (
            <>
              <RefreshCw className="spin" size={16} />
              {t("processing")}
            </>
          ) : (
            <>
              <Zap size={16} />
              {passed ? t("connectionPassed") : t("testConnection")}
            </>
          )}
        </button>
      </div>
    </div>
  );
}

function AboutView({ t }: { t: (key: string) => string }) {
  return (
    <section className="page about-page">
      <div className="about-hero">
        <span className="eyebrow">VERILECTURE / 0.3.0-ALPHA.1</span>
        <h1>{t("aboutTitle")}</h1>
        <p>{t("aboutLead")}</p>
        <div className="about-seal">
          课<br />
          <small>TRACE</small>
        </div>
      </div>
      <div className="about-grid">
        <div>
          <span className="eyebrow">{t("maintainer")}</span>
          <h2>xiajiadi</h2>
        </div>
        <div>
          <span className="eyebrow">{t("license")}</span>
          <h2>{t("openSource")}</h2>
        </div>
        <div className="about-copy">
          <p>{t("privacyLocal")}</p>
          <p>{t("privacyCloud")}</p>
          <p className="attribution">{t("attribution")}</p>
        </div>
      </div>
      <section className="about-licenses" aria-labelledby="third-party-licenses-title">
        <div>
          <span className="eyebrow">{t("license")}</span>
          <h2 id="third-party-licenses-title">{t("thirdPartyLicensesTitle")}</h2>
          <p>{t("thirdPartyLicensesLead")}</p>
        </div>
        <ul>
          <li>{t("licenseQwen")}</li>
          <li>{t("licenseFun")}</li>
          <li>{t("licenseRuntime")}</li>
          <li>{t("licenseTauri")}</li>
          <li>{t("licenseFfmpeg")}</li>
          <li>{t("licenseMeetily")}</li>
        </ul>
      </section>
    </section>
  );
}

function EmptyState({
  icon,
  title,
  lead,
}: {
  icon: ReactNode;
  title: string;
  lead: string;
}) {
  return (
    <div className="empty-state">
      <div className="empty-icon">{icon}</div>
      <h3>{title}</h3>
      {lead ? <p>{lead}</p> : null}
    </div>
  );
}

function TutorialModal({
  close,
  t,
}: {
  close: () => void;
  t: (key: string) => string;
}) {
  const [step, setStep] = useState(0);
  const pages = [
    {
      icon: Upload,
      title: t("tutorialImportTitle"),
      lead: t("tutorialImportLead"),
    },
    {
      icon: Headphones,
      title: t("tutorialTranscriptTitle"),
      lead: t("tutorialTranscriptLead"),
    },
    {
      icon: Library,
      title: t("tutorialLexiconTitle"),
      lead: t("tutorialLexiconLead"),
    },
    {
      icon: Sparkles,
      title: t("tutorialPointsTitle"),
      lead: t("tutorialPointsLead"),
    },
  ];
  const current = pages[step];
  const Icon = current.icon;
  return (
    <div className="onboarding-backdrop">
      <div
        className="tutorial-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="tutorial-title"
      >
        <div className="tutorial-top">
          <div>
            <span className="eyebrow">{t("tutorialTitle")}</span>
            <h1 id="tutorial-title">{current.title}</h1>
          </div>
          <button className="text-button" onClick={close}>
            {t("skip")}
          </button>
        </div>
        <div className="tutorial-progress" aria-hidden="true">
          {pages.map((_, index) => (
            <i className={index <= step ? "active" : ""} key={index} />
          ))}
        </div>
        <div className="tutorial-body">
          <div className="tutorial-icon">
            <Icon size={30} />
          </div>
          <p>{t("tutorialLead")}</p>
          <p className="tutorial-detail">{current.lead}</p>
        </div>
        <div className="tutorial-footer">
          <button
            className="text-button"
            onClick={() => setStep((value) => Math.max(0, value - 1))}
            disabled={step === 0}
          >
            {t("back")}
          </button>
          <button
            className="primary-button"
            onClick={() =>
              step === pages.length - 1
                ? close()
                : setStep((value) => value + 1)
            }
          >
            {step === pages.length - 1 ? t("finish") : t("tutorialNext")}{" "}
            <ArrowRight size={16} />
          </button>
        </div>
      </div>
    </div>
  );
}

function OnboardingModal({
  snapshot,
  setSnapshot,
  close,
  locale,
  setLocale,
  t,
  setToast,
}: {
  snapshot: AppSnapshot;
  setSnapshot: Dispatch<SetStateAction<AppSnapshot>>;
  close: () => void | Promise<void>;
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
  setToast: (message: string) => void;
}) {
  const [step, setStep] = useState(0);
  const [selectedModel, setSelectedModel] = useState<ModelId | null>(
    snapshot.selectedModelId ??
      snapshot.models.find((model) => model.recommended)?.id ??
      null,
  );
  const [scanning, setScanning] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const [provider, setProvider] = useState<ProviderConfig | null>(
    snapshot.provider,
  );
  const [providerPassed, setProviderPassed] = useState(
    Boolean(snapshot.provider?.tested),
  );
  useEffect(() => {
    let active = true;
    let dispose: (() => void) | undefined;
    void subscribeInstallProgress((event) => {
      if (active && event.modelId === selectedModel) setProgress(event);
    }).then((unlisten) => {
      if (active) dispose = unlisten;
      else unlisten();
    });
    return () => {
      active = false;
      dispose?.();
    };
  }, [selectedModel]);
  const scan = async () => {
    setScanning(true);
    try {
      const hardware = await scanHardware();
      const models = await getModelCatalog();
      setSnapshot((current) => ({ ...current, hardware, models }));
      const recommendation =
        models.find((model) => model.recommended && model.supported) ??
        models.find((model) => model.supported);
      setSelectedModel(recommendation?.id ?? null);
      setStep(2);
    } catch {
      if (!isTauri) {
        const hardware = demoHardware(locale);
        const models = demoModels(locale);
        setSnapshot((current) => ({ ...current, hardware, models }));
        setSelectedModel(models.find((model) => model.supported)?.id ?? null);
        setStep(2);
      } else {
        setToast(t("errorGeneric"));
      }
    } finally {
      setScanning(false);
    }
  };
  const chosen = snapshot.models.find((model) => model.id === selectedModel);
  const install = async () => {
    if (!selectedModel || !chosen?.supported) return;
    if (chosen.status === "ready") {
      if (isTauri) {
        try {
          await selectModel(selectedModel);
        } catch {
          setToast(t("errorGeneric"));
          return;
        }
      }
      setSnapshot((current) => ({
        ...current,
        selectedModelId: selectedModel,
      }));
      setStep(3);
      return;
    }
    setInstalling(true);
    try {
      if (isTauri) {
        await installModel(selectedModel);
      } else {
        await new Promise((resolve) => setTimeout(resolve, 850));
      }
      setSnapshot((current) => ({
        ...current,
        selectedModelId: selectedModel,
        models: current.models.map((model) =>
          model.id === selectedModel ? { ...model, status: "ready" } : model,
        ),
      }));
      setProgress({
        modelId: selectedModel,
        stage: "ready",
        fileName: "",
        downloadedBytes: chosen.downloadBytes,
        totalBytes: chosen.downloadBytes,
        speedBytesPerSecond: 0,
        message: "READY",
      });
      setStep(3);
    } catch (error) {
      setToast(modelInstallErrorMessage(error, t));
    } finally {
      setInstalling(false);
    }
  };
  const pause = async () => {
    if (!selectedModel) return;
    try {
      await pauseModel(selectedModel);
      setProgress((current) =>
        current
          ? { ...current, stage: "paused", message: t("installPaused") }
          : current,
      );
    } catch {
      setToast(t("errorGeneric"));
    }
  };
  const resume = async () => {
    if (!selectedModel) return;
    try {
      await resumeModel(selectedModel);
      setProgress((current) =>
        current
          ? { ...current, stage: "downloading", message: t("processing") }
          : current,
      );
    } catch {
      setToast(t("errorGeneric"));
    }
  };
  const cancel = async () => {
    if (!selectedModel) return;
    try {
      await cancelModel(selectedModel);
      setProgress((current) =>
        current
          ? { ...current, stage: "error", message: t("installCancelled") }
          : current,
      );
    } catch {
      setToast(t("errorGeneric"));
    }
  };
  return (
    <div className="onboarding-backdrop">
      <div className="onboarding-modal">
        <div className="onboarding-top">
          <div className="brand-lockup compact">
            <div className="brand-symbol">课</div>
            <div>
              <div className="brand-name">{t("brand")}</div>
              <div className="brand-sub">{t("brandSub")}</div>
            </div>
          </div>
          <div className="onboarding-actions">
            <button
              className="language-chip"
              onClick={() => setLocale(locale === "zh-CN" ? "en-US" : "zh-CN")}
            >
              {locale === "zh-CN" ? "EN" : "中"}
            </button>
            {step > 0 && step < 5 ? <span>{step} / 4</span> : null}
          </div>
        </div>
        <div className="onboarding-body">
          {step === 0 ? (
            <div className="onboarding-intro">
              <div className="onboarding-art">
                <span>{t("audioLabel")}</span>
                <i />
                <span>{t("pointsLabel")}</span>
              </div>
              <span className="eyebrow">{t("welcomeEyebrow")}</span>
              <h1>{t("onboardingWelcome")}</h1>
              <p>{t("onboardingWelcomeLead")}</p>
              <div className="privacy-list">
                <span>
                  <ShieldCheck size={16} />
                  {t("privacyLocal")}
                </span>
                <span>
                  <LockKeyhole size={16} />
                  {t("privacyCloud")}
                </span>
                <span>
                  <FileAudio size={16} />
                  {t("sourceNotDeleted")}
                </span>
              </div>
              <button
                className="primary-button onboarding-next"
                onClick={() => {
                  setStep(1);
                  void scan();
                }}
              >
                {t("continue")} <ArrowRight size={17} />
              </button>
            </div>
          ) : null}
          {step === 1 ? (
            <div className="onboarding-step">
              <StepHeader
                number="01"
                title={t("hardwareScan")}
                lead={t("hardwareScanLead")}
              />
              {snapshot.hardware?.scannedAt ? (
                <HardwareSummary hardware={snapshot.hardware} t={t} />
              ) : null}
              <button
                className="primary-button onboarding-next"
                onClick={scan}
                disabled={scanning}
              >
                {scanning ? (
                  <>
                    <RefreshCw className="spin" size={17} />
                    {t("scanning")}
                  </>
                ) : (
                  <>
                    <MonitorCog size={17} />
                    {t("hardwareScan")}
                  </>
                )}
              </button>
            </div>
          ) : null}
          {step === 2 ? (
            <div className="onboarding-step">
              <StepHeader
                number="02"
                title={t("downloadModel")}
                lead={t("modelInstallLead")}
              />
              <div className="model-choices">
                {modelOrder.map((id) => {
                  const model = snapshot.models.find((item) => item.id === id);
                  return model ? (
                    <ModelChoice
                      key={id}
                      model={model}
                      selected={selectedModel === id}
                      onSelect={() => model.supported && setSelectedModel(id)}
                      t={t}
                    />
                  ) : null;
                })}
              </div>
              {chosen?.reason && !chosen.supported ? (
                <div className="inline-warning">
                  <CircleHelp size={16} />
                  {chosen.reason}
                </div>
              ) : null}
              {progress &&
              progress.modelId === selectedModel &&
              progress.stage !== "ready" ? (
                <ModelInstallProgress
                  progress={progress}
                  installing={installing}
                  onPause={pause}
                  onResume={resume}
                  onCancel={cancel}
                  t={t}
                />
              ) : null}
              <button
                className="primary-button onboarding-next"
                onClick={() => void install()}
                disabled={
                  !chosen?.supported ||
                  installing ||
                  progress?.stage === "paused"
                }
              >
                {installing ? (
                  <>
                    <RefreshCw className="spin" size={17} />
                    {t("processing")}
                  </>
                ) : (
                  <>
                    <Download size={17} />
                    {chosen?.status === "ready"
                      ? t("continue")
                      : t("installAndContinue")}
                  </>
                )}
              </button>
            </div>
          ) : null}
          {step === 3 ? (
            <div className="onboarding-step">
              <StepHeader
                number="03"
                title={t("providerSetup")}
                lead={t("providerSetupLead")}
              />
              <OnboardingProvider
                provider={provider}
                setProvider={setProvider}
                passed={providerPassed}
                setPassed={setProviderPassed}
                t={t}
                setSnapshot={setSnapshot}
                setToast={setToast}
              />
              <button
                className="primary-button onboarding-next"
                disabled={!providerPassed}
                onClick={() => setStep(4)}
              >
                {t("continue")} <ArrowRight size={17} />
              </button>
            </div>
          ) : null}
          {step === 4 ? (
            <div className="onboarding-intro finish-step">
              <div className="finish-check">
                <Check size={28} />
              </div>
              <span className="eyebrow">{t("readyEyebrow")}</span>
              <h1>{t("setupComplete")}</h1>
              <p>{t("setupCompleteLead")}</p>
              <div className="finish-summary">
                <span>
                  <strong>
                    {chosen?.name ??
                      modelLabel(snapshot.selectedModelId, snapshot.models)}
                  </strong>
                  {t("runtime")}
                </span>
                <span>
                  <strong>{provider?.modelId ?? "—"}</strong>
                  {t("provider")}
                </span>
              </div>
              <button
                className="primary-button onboarding-next"
                onClick={close}
              >
                {t("finish")} <ArrowRight size={17} />
              </button>
            </div>
          ) : null}
        </div>
        <div className="onboarding-footer">
          <span>{t("aiNotice")}</span>
          {step === 0 ? (
            <button
              className="text-button"
              onClick={() => {
                setStep(1);
                void scan();
              }}
            >
              {t("skip")}
            </button>
          ) : step > 0 && step < 4 ? (
            <button className="text-button" onClick={() => setStep(step - 1)}>
              <ArrowLeft size={14} />
              {t("back")}
            </button>
          ) : null}
        </div>
      </div>
    </div>
  );
}

function ModelInstallProgress({
  progress,
  installing,
  onPause,
  onResume,
  onCancel,
  t,
}: {
  progress: InstallProgress;
  installing: boolean;
  onPause: () => Promise<void>;
  onResume: () => Promise<void>;
  onCancel: () => Promise<void>;
  t: (key: string) => string;
}) {
  const percent = progress.totalBytes
    ? Math.min(100, (progress.downloadedBytes / progress.totalBytes) * 100)
    : 0;
  const status = modelProgressStage(progress.stage, t);
  return (
    <div className="install-progress-panel" aria-live="polite">
      <div className="install-progress-heading">
        <strong>{t("installProgress")}</strong>
        <span>{status}</span>
      </div>
      <div className="progress-track">
        <i style={{ width: `${percent}%` }} />
      </div>
      <div className="install-progress-meta">
        <span>
          {progress.fileName
            ? `${t("installFile")}: ${progress.fileName}`
            : modelProgressMessage(progress.message, t)}
        </span>
        <span>
          {formatBytes(progress.downloadedBytes)} /{" "}
          {formatBytes(progress.totalBytes)}
        </span>
      </div>
      <div className="install-progress-controls">
        {progress.stage === "paused" ? (
          <button className="quiet-button" onClick={() => void onResume()}>
            <Download size={15} />
            {t("resume")}
          </button>
        ) : (
          <button
            className="quiet-button"
            onClick={() => void onPause()}
            disabled={!installing}
          >
            <Pause size={15} />
            {t("pause")}
          </button>
        )}
        <button
          className="text-button"
          onClick={() => void onCancel()}
          disabled={!installing}
        >
          {t("cancel")}
        </button>
      </div>
    </div>
  );
}

function StepHeader({
  number,
  title,
  lead,
}: {
  number: string;
  title: string;
  lead: string;
}) {
  return (
    <div className="step-header">
      <span className="step-number">{number}</span>
      <div>
        <h1>{title}</h1>
        <p>{lead}</p>
      </div>
    </div>
  );
}
function HardwareSummary({
  hardware,
  t,
}: {
  hardware: HardwareProfile;
  t: (key: string) => string;
}) {
  const osVersion = hardware.osVersion.trim();
  const displayOsVersion =
    osVersion &&
    !osVersion.includes("\uFFFD") &&
    !osVersion.toLowerCase().startsWith("microsoft windows [")
      ? osVersion
      : hardware.os.toLowerCase() === "windows"
        ? "Windows"
        : hardware.os;
  return (
    <div className="hardware-summary">
      <Fact
        label={t("operatingSystem")}
        value={displayOsVersion}
      />
      <Fact label="CPU" value={hardware.cpuName} />
      <Fact label="RAM" value={formatBytes(hardware.totalRamBytes)} />
      <Fact label="GPU" value={hardware.gpuName ?? t("cpu")} />
      <Fact
        label="CUDA"
        value={hardware.cudaSmokeTest ? t("cudaReady") : t("cudaUnavailable")}
      />
      <Fact
        label={t("proxy")}
        value={
          hardware.proxyConfigured
            ? t("proxyConfigured")
            : t("proxyNotConfigured")
        }
      />
    </div>
  );
}
function ModelChoice({
  model,
  selected,
  onSelect,
  t,
}: {
  model: ModelOption;
  selected: boolean;
  onSelect: () => void;
  t: (key: string) => string;
}) {
  return (
    <button
      className={`model-choice ${selected ? "selected" : ""} ${!model.supported ? "disabled" : ""}`}
      onClick={onSelect}
      disabled={!model.supported}
    >
      <div className="model-choice-radio">
        {selected ? <Check size={14} /> : null}
      </div>
      <div className="model-choice-copy">
        <div className="model-choice-title">
          <strong>{model.name}</strong>
          {model.recommended ? <span>{t("recommended")}</span> : null}
        </div>
        <p>{modelDescription(model.id, t)}</p>
        <small>
          {model.runtime} · {formatBytes(model.downloadBytes)} {t("download")} ·{" "}
          {model.requiresAligner ? t("forcedAlignerTimestamp") : t("timestamp")}
        </small>
        {!model.supported || model.reason.includes("will be downloaded") ? (
          <em>{modelReason(model, t)}</em>
        ) : null}
      </div>
    </button>
  );
}

function OnboardingProvider({
  provider,
  setProvider,
  passed,
  setPassed,
  t,
  setSnapshot,
  setToast,
}: {
  provider: ProviderConfig | null;
  setProvider: (provider: ProviderConfig) => void;
  passed: boolean;
  setPassed: (passed: boolean) => void;
  t: (key: string) => string;
  setSnapshot: Dispatch<SetStateAction<AppSnapshot>>;
  setToast: (message: string) => void;
}) {
  const [apiKey, setApiKey] = useState("");
  const [consent, setConsent] = useState(provider?.consentGranted ?? false);
  const [testing, setTesting] = useState(false);
  const current = provider ?? {
    provider: "OpenAI",
    protocol: "openai_compatible" as const,
    baseUrl: "https://api.openai.com/v1",
    modelId: "gpt-4o-mini",
    timeoutSeconds: 60,
    configured: false,
    tested: false,
    consentGranted: false,
  };
  const update = (patch: Partial<ProviderConfig>) =>
    setProvider({ ...current, ...patch });
  const run = async () => {
    if (!consent) {
      setToast(t("dataConsentRequired"));
      return;
    }
    setTesting(true);
    try {
      const config = {
        ...current,
        configured: true,
        tested: false,
        consentGranted: true,
      };
      const result = await testProvider(config, apiKey);
      if (!result.ok) throw new Error(result.message);
      const ready = { ...config, tested: true };
      if (isTauri) await saveProvider(ready, apiKey);
      setProvider(ready);
      setPassed(true);
      setSnapshot((snapshot) => ({ ...snapshot, provider: ready }));
    } catch {
      if (!isTauri && apiKey.trim()) {
        const ready = {
          ...current,
          configured: true,
          tested: true,
          consentGranted: true,
        };
        setProvider(ready);
        setPassed(true);
        setSnapshot((snapshot) => ({ ...snapshot, provider: ready }));
        setToast(t("browserPreview"));
      } else {
        setPassed(false);
        setToast(t("connectionFailed"));
      }
    } finally {
      setTesting(false);
    }
  };
  return (
    <div className="onboarding-provider">
      <div className="form-grid">
        <label>
          {t("provider")}
          <select
            className="field-select"
            value={current.provider}
            onChange={(event) => {
              const value = event.target.value;
              update({
                provider: value,
                protocol: providerProtocol(value),
                baseUrl: providerBaseUrl(value) || current.baseUrl,
              });
            }}
          >
            {providerOptions.map(([value, key]) => (
              <option value={value} key={value}>
                {t(key)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("modelId")}
          <input
            className="text-field"
            value={current.modelId}
            onChange={(event) => update({ modelId: event.target.value })}
          />
        </label>
        <label className="wide">
          {t("baseUrl")}
          <input
            className="text-field"
            value={current.baseUrl}
            onChange={(event) => update({ baseUrl: event.target.value })}
          />
        </label>
        <label className="wide">
          {t("apiKey")}
          <input
            className="text-field"
            type="password"
            value={apiKey}
            onChange={(event) => setApiKey(event.target.value)}
            placeholder={t("apiKeyPlaceholder")}
          />
        </label>
      </div>
      <label className="check-label">
        <input
          type="checkbox"
          checked={consent}
          onChange={(event) => setConsent(event.target.checked)}
        />
        {t("dataConsent")}
      </label>
      <button
        className="secondary-button test-provider"
        onClick={run}
        disabled={testing || !apiKey || !current.modelId}
      >
        {testing ? (
          <>
            <RefreshCw className="spin" size={15} />
            {t("processing")}
          </>
        ) : passed ? (
          <>
            <Check size={15} />
            {t("connectionPassed")}
          </>
        ) : (
          <>
            <Zap size={15} />
            {t("testConnection")}
          </>
        )}
      </button>
    </div>
  );
}
