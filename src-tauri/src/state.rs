use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

/// 一个运行中任务的进程控制句柄。真正的 `tokio::process::Child` 由 runner 的
/// monitor 任务独占持有，这里只保留按 pid 杀进程所需的信息与共享的 killed 标记。
pub struct ChildHandle {
    pub pid: u32,
    pub killed: Arc<AtomicBool>,
}

pub struct AppState {
    pub running: Arc<Mutex<HashMap<String, ChildHandle>>>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            running: Arc::new(Mutex::new(HashMap::new())),
            data_dir,
        }
    }

    pub fn running_count(&self) -> usize {
        self.running.lock().map(|m| m.len()).unwrap_or(0)
    }
}
