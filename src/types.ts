// 与后端 src-tauri/src/models.rs 一一对应（camelCase 契约）

export type ShellKind = "powershell" | "cmd" | "bash" | "sh";

export type ExecType = "command" | "file";

export type RunStatus =
  | "running"
  | "success"
  | "failed"
  | "killed"
  | "timeout"
  | "interrupted"
  | "error";

export interface ShellConfig {
  id: string;
  name: string;
  kind: ShellKind;
  exe: string;
  args: string[];
  builtin: boolean;
}

export interface Script {
  id: string;
  name: string;
  description: string;
  shellId: string; // 引用 ShellConfig.id
  execType: ExecType;
  command: string;
  cwd: string | null;
  env: Record<string, string>;
  timeoutSec: number;
  createdAt: number;
  updatedAt: number;
}

export interface RunRecord {
  id: string;
  scriptId: string | null;
  scriptName: string;
  shellId: string;
  shellName: string;
  command: string;
  cwd: string | null;
  status: RunStatus;
  exitCode: number | null;
  startedAt: number;
  finishedAt: number | null;
  logPath: string;
}

export interface RunInput {
  scriptId: string | null;
  scriptName: string;
  shellId: string;
  command: string;
  execType: ExecType;
  cwd: string | null;
  env: Record<string, string>;
  timeoutSec: number;
}

export type OutputEvent =
  | { type: "start"; runId: string; pid: number }
  | { type: "stdout"; runId: string; data: string }
  | { type: "stderr"; runId: string; data: string }
  | { type: "exit"; runId: string; code: number | null };
