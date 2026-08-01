import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  OutputEvent,
  RunInput,
  RunRecord,
  Script,
  ShellConfig,
} from "./types";

export const api = {
  listScripts: () => invoke<Script[]>("list_scripts"),

  saveScript: (script: Script) => invoke<Script>("save_script", { script }),

  deleteScript: (id: string) => invoke<void>("delete_script", { id }),

  listShells: () => invoke<ShellConfig[]>("list_shells"),

  saveShell: (shell: ShellConfig) => invoke<ShellConfig>("save_shell", { shell }),

  deleteShell: (id: string) => invoke<void>("delete_shell", { id }),

  runScript: (input: RunInput, onEvent: (e: OutputEvent) => void) => {
    const channel = new Channel<OutputEvent>();
    channel.onmessage = onEvent;
    return invoke<RunRecord>("run_script", { input, channel });
  },

  killRun: (runId: string) => invoke<void>("kill_run", { runId }),

  listRuns: () => invoke<RunRecord[]>("list_runs"),

  getRunLog: (runId: string) => invoke<string>("get_run_log", { runId }),

  clearHistory: (scriptId?: string) =>
    invoke<void>("clear_history", { scriptId: scriptId ?? null }),

  runningCount: () => invoke<number>("running_count"),
};
