# 脚本管理后台 — 开发文档

> 基于 **Tauri 2 + Vue 3 + TypeScript** 的本地桌面脚本管理工具:管理、执行、监控脚本。

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Vue 3（`<script setup>`）、Vue Router（hash 模式）、手写 UI 组件（无组件库） |
| 后端 | Rust + tokio（异步进程执行） |
| 通信 | Tauri `invoke` + `Channel<OutputEvent>` 事件流 |
| 存储 | 应用数据目录下 JSON 文件 + 日志文件 |
| 包管理 | bun（`bun.lock`） |

## 目录结构

```
src/                        # 前端
  api.ts                    # invoke 封装（12 个命令）
  types.ts                  # IPC 契约类型（camelCase，与 models.rs 一一对应）
  store.ts                  # 全局 reactive store（脚本/Shell/历史/实时任务）
  router/index.ts           # 路由：/scripts /console /history /shells
  components/
    ScriptsView.vue         # 脚本库（列表 + 新增/编辑表单 + 快速执行）
    ShellManagementView.vue # Shell 管理
    RunConsoleView.vue      # 运行控制台（任务卡片 + 实时日志）
    HistoryView.vue         # 运行历史（筛选 + 日志回看 + 重跑）
    ui/                     # Button/Modal/Select/Table/Tag/Toast 等基础组件
  styles/                   # tokens.css 设计 token、base.css 基础样式
src-tauri/src/              # 后端
  lib.rs                    # 启动装配 + 命令注册
  models.rs                 # 数据模型 + serde 契约
  detect.rs                 # 启动时检测本机 Shell
  storage.rs                # JSON 落盘 + 日志追加读写
  runner.rs                 # 进程派生/流式输出/终止/超时
  state.rs                  # AppState（运行中任务表）
src-tauri/tests/commands.rs # 命令级集成测试
```

## 架构与数据流

```
Vue 管理界面（store 驱动）
   │  invoke + Channel 事件流
   ▼
Rust 后端（runner 派生子进程）
   ▼
应用数据目录（app_data_dir）
   ├─ scripts.json    脚本库
   ├─ shells.json     Shell 配置（内置 + 自定义）
   ├─ history.json    运行历史（上限 500 条，新在前）
   └─ logs/<runId>.log 每次运行的完整日志（实时追加）
```

启动流程（`lib.rs` setup）：

1. 创建数据目录
2. `detect::detect_shells()` 检测系统 Shell → `merge_detected` 合并进 `shells.json`（按 exe 去重，保留自定义项）
3. `migrate_running_on_boot` 把上次遗留的 `running` 记录标记为 `interrupted`
4. 注册 `AppState`

## 数据模型与 IPC 契约

- 全部字段 **camelCase** 序列化；`ShellKind` 显式覆写为小写（`powershell`/`cmd`/`bash`/`sh`），保证 `shells.json` 落盘稳定。
- `OutputEvent` 为 tagged enum：`start`（runId/pid）、`stdout`/`stderr`（data）、`exit`（code）。
- 状态枚举：`running / success / failed / killed / timeout / interrupted / error`。

### Tauri 命令

| 命令 | 说明 |
|---|---|
| `list_scripts` / `save_script` / `delete_script` | 脚本 CRUD（`id` 为空则新建，否则更新） |
| `list_shells` / `save_shell` / `delete_shell` | Shell CRUD（内置不可删、被脚本引用不可删；保存时按 exe 推断 kind） |
| `run_script` | 传入 `RunInput` + `Channel<OutputEvent>`，返回 `RunRecord` |
| `kill_run` | 按 runId 终止进程树 |
| `list_runs` / `get_run_log` / `clear_history` | 历史读取、日志全文回看、清空（可按脚本，连带删除日志文件） |
| `running_count` | 运行中任务数（侧边栏徽标） |

## 核心流程

**运行脚本**（`runner.rs::spawn`）：

1. 按 `shell_id` 查 Shell 配置，`resolve_command` 把输入解析为进程命令 —— **命令/文件路径/工作目录一律走 args 数组，不做字符串拼接**，避免注入与空格问题
2. 生成 `runId`，历史写 `running` 记录，注册 `ChildHandle { pid, killed }`
3. 发 `start` 事件 → 两个 `pipe_reader` 任务分别读 stdout/stderr：**追加写磁盘日志 + 经 Channel 推给前端**。输出按 **UTF-8 优先、遇到非法字节回退 GBK** 解码后再落盘（Windows 中文控制台如 java/cmd 常按 GBK/CP936 输出，硬按 UTF-8 解会乱码）
4. `monitor_task` 等待子进程（`timeout_sec` 后杀进程树），回收 reader 保证日志完整，回写状态并发 `exit` 事件

**终止任务**：置 `killed` 标记 → Windows `taskkill /PID <pid> /T /F`（杀整棵树），Unix 对进程组 `SIGTERM` + 800ms 后 `SIGKILL`。状态由 monitor 统一回写为 `killed`。

**前端实时日志**：`store.ts` 中每个 `LiveRun` 仅缓存最近 `MAX_LOG_LINES = 2000` 行；`finalize` 按退出码 + killed 标记推导结束状态。

## 开发与构建

```bash
bun install                # 安装依赖
bun run tauri dev          # 启动开发（Vite 固定端口 1420 + Rust 编译）
bun run dev                # 仅前端（浏览器调试 UI）
bun run build              # 类型检查（vue-tsc）+ 前端构建
bun run tauri build        # 打包桌面应用
```

Rust 侧单独测试：

```bash
cd src-tauri
cargo test                 # 单元测试（runner/storage/models/detect）+ 集成测试（tests/commands.rs）
```

## 测试约定

- Rust 单元测试覆盖：命令解析（各 Shell/类型）、日志落盘与 500 条上限淘汰、清空历史联删日志、Shell 增删约束、启动中断迁移、`OutputEvent`/`ShellKind` 的 camelCase 序列化契约、输出解码自动识别（UTF-8/GBK，含跨 chunk 拆分字符）。
- 集成测试（`tests/commands.rs`）用 `tauri::test` mock runtime 全链路调用命令（Cargo.toml 中需显式声明 test target + Windows Common-Controls v6 清单）。
- 修改 IPC 契约时，务必同步更新 `models.rs`、`src/types.ts` 与序列化回归测试。
