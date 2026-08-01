use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShellKind {
    PowerShell,
    Cmd,
    Bash,
    Sh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ExecType {
    #[default]
    Command,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunStatus {
    Running,
    Success,
    Failed,
    Killed,
    Timeout,
    Interrupted,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub shell_id: String,
    #[serde(default)]
    pub exec_type: ExecType,
    pub command: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub timeout_sec: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellConfig {
    pub id: String,
    pub name: String,
    pub kind: ShellKind,
    pub exe: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub builtin: bool,
}

impl ShellConfig {
    pub fn builtin(kind: ShellKind, id: &str, name: &str, exe: &str, args: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            exe: exe.to_string(),
            args,
            builtin: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    pub id: String,
    pub script_id: Option<String>,
    pub script_name: String,
    pub shell_id: String,
    pub shell_name: String,
    pub command: String,
    pub cwd: Option<String>,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub log_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInput {
    pub script_id: Option<String>,
    pub script_name: String,
    pub shell_id: String,
    pub command: String,
    #[serde(default)]
    pub exec_type: ExecType,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OutputEvent {
    Start { run_id: String, pid: u32 },
    Stdout { run_id: String, data: String },
    Stderr { run_id: String, data: String },
    Exit { run_id: String, code: Option<i32> },
}
