import { reactive } from "vue";
import { api } from "./api";
import type {
  OutputEvent,
  RunInput,
  RunRecord,
  RunStatus,
  Script,
  ShellConfig,
} from "./types";

const TOAST_MS = 2500;
export const MAX_LOG_LINES = 2000;

export interface LogLine {
  stream: "out" | "err";
  text: string;
}

export interface LiveRun {
  record: RunRecord;
  pid: number;
  logs: LogLine[];
  paused: boolean;
  killed: boolean;
}

export type ToastType = "success" | "error" | "info";

export interface ToastItem {
  id: number;
  msg: string;
  type: ToastType;
}

export interface Store {
  scripts: Script[];
  shells: ShellConfig[];
  runs: RunRecord[];
  live: Record<string, LiveRun>;
  toasts: ToastItem[];
  readonly runningCount: number;
  readonly defaultShellId: string;
  toast(msg: string, type?: ToastType): void;
  startRun(input: RunInput): Promise<RunRecord | null>;
  killRun(runId: string): Promise<void>;
  clearFinished(): void;
  refreshScripts(): Promise<void>;
  refreshShells(): Promise<void>;
  refreshHistory(): Promise<void>;
}

let toastSeq = 0;

let store: Store;
export { store };

store = reactive<Store>({
  scripts: [],
  shells: [],
  runs: [],
  live: {},
  toasts: [],

  get runningCount() {
    // 徽标语义为「运行中任务数」：已结束但尚未清除的卡片不计入
    return Object.values(store.live).filter((r) => r.record.status === "running").length;
  },

  get defaultShellId() {
    // 新建脚本默认：第一个内置 shell，否则第一个
    return (store.shells.find((s) => s.builtin) ?? store.shells[0])?.id ?? "";
  },

  toast(msg: string, type: ToastType = "info") {
    const id = ++toastSeq;
    store.toasts.push({ id, msg, type });
    window.setTimeout(() => {
      const idx = store.toasts.findIndex((t) => t.id === id);
      if (idx >= 0) store.toasts.splice(idx, 1);
    }, TOAST_MS);
  },

  async startRun(input: RunInput): Promise<RunRecord | null> {
    let record: RunRecord;
    try {
      record = await api.runScript(input, (ev) => handleOutput(store, ev));
    } catch (e) {
      store.toast(errorMessage(e), "error");
      return null;
    }
    ensureLive(store, record);
    const { router } = await import("./router");
    await router.push("/console");
    return record;
  },

  async killRun(runId: string): Promise<void> {
    const live = store.live[runId];
    if (live && live.record.status === "running") live.killed = true;
    try {
      await api.killRun(runId);
    } catch (e) {
      if (live) live.killed = false;
      store.toast(errorMessage(e), "error");
    }
  },

  clearFinished() {
    for (const id of Object.keys(store.live)) {
      if (store.live[id].record.status !== "running") delete store.live[id];
    }
  },

  async refreshScripts() {
    try {
      store.scripts = await api.listScripts();
    } catch (e) {
      store.toast(errorMessage(e), "error");
    }
  },

  async refreshShells() {
    try {
      store.shells = await api.listShells();
    } catch (e) {
      store.toast(errorMessage(e), "error");
    }
  },

  async refreshHistory() {
    try {
      store.runs = await api.listRuns();
    } catch (e) {
      store.toast(errorMessage(e), "error");
    }
  },
});

function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

function ensureLive(store: Store, record: RunRecord, pid: number | null = null): LiveRun {
  const existing = store.live[record.id];
  if (existing) {
    if (pid !== null) existing.pid = pid;
    existing.record = record;
    return existing;
  }
  const live: LiveRun = {
    record: { ...record },
    pid: pid ?? 0,
    logs: [],
    paused: false,
    killed: false,
  };
  store.live[record.id] = live;
  return live;
}

function handleOutput(store: Store, ev: OutputEvent) {
  if (ev.type === "start") {
    const existing = store.live[ev.runId];
    if (existing) {
      existing.pid = ev.pid;
      return;
    }
    // Start 事件可能先于 invoke 返回：先占位，等 ensureLive 补全 record
    ensureLive(
      store,
      {
        id: ev.runId,
        scriptId: null,
        scriptName: "",
        shellId: "",
        shellName: "",
        command: "",
        cwd: null,
        status: "running",
        exitCode: null,
        startedAt: Date.now(),
        finishedAt: null,
        logPath: "",
      },
      ev.pid,
    );
    return;
  }
  const live = store.live[ev.runId];
  if (!live) return;
  if (ev.type === "stdout") pushLines(live, "out", ev.data);
  else if (ev.type === "stderr") pushLines(live, "err", ev.data);
  else finalize(store, live, ev.code);
}

function pushLines(live: LiveRun, stream: LogLine["stream"], data: string) {
  // 整批组装后一次 push：reactive 数组 push(...items) 只触发一次，避免逐行触发
  // 响应式更新；\r 快路径（绝大多数行没有 CR 时跳过正则）。
  const batch: LogLine[] = [];
  for (const raw of data.split("\n")) {
    const text = raw.endsWith("\r") ? raw.replace(/\r+$/, "") : raw;
    if (text.length === 0) continue;
    batch.push({ stream, text });
  }
  if (batch.length === 0) return;
  live.logs.push(...batch);
  if (live.logs.length > MAX_LOG_LINES) {
    live.logs.splice(0, live.logs.length - MAX_LOG_LINES);
  }
}

function finalize(store: Store, live: LiveRun, code: number | null) {
  const rec = live.record;
  rec.finishedAt = Date.now();
  rec.exitCode = code;
  if (live.killed) rec.status = "killed";
  else if (code === 0) rec.status = "success";
  else if (code !== null) rec.status = "failed";
  else rec.status = "timeout";
  void store.refreshHistory();
}

// 状态 → 标签/语义色（Tag.vue 按 tone 映射 CSS 变量）
export const statusMeta: Record<RunStatus, { label: string; tone: string }> = {
  running: { label: "运行中", tone: "running" },
  success: { label: "成功", tone: "success" },
  failed: { label: "失败", tone: "failed" },
  killed: { label: "被终止", tone: "killed" },
  timeout: { label: "超时", tone: "timeout" },
  interrupted: { label: "中断", tone: "interrupted" },
  error: { label: "错误", tone: "failed" },
};

export function shellLabel(id: string): string {
  return store.shells.find((s) => s.id === id)?.name ?? id;
}
