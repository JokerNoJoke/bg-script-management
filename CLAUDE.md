# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Tauri 2 + Vue 3 + TypeScript 本地桌面工具:管理、执行、监控脚本(单用户、无鉴权、以当前用户权限运行)。权威架构/契约/生命周期文档见 **DEVELOPMENT.md**(仓库根目录) —— 改动 IPC 或 runner 前先读它。

## Docs 分工与修改时机

三份文档按受众分工,**避免重复**。改代码时判断变更属于哪一层,只更新对应文档:

| 文档 | 受众 | 什么时候改 |
|---|---|---|
| **README.md** | 用户 | 功能增减、安装/构建方式、技术栈变更 —— 用户可见的变化;内部重构不动它 |
| **DEVELOPMENT.md** | 开发者(权威手册) | 实现细节:新/改 Tauri 命令、数据模型字段、存储结构、runner 流程、测试约定。**随代码走,唯一权威** |
| **CLAUDE.md** | Claude AI | 新增/消失的"静默出错"不变量、前端 gotcha、编辑约束 —— 只在不变量改变时改,比开发文档稳定 |

判断规则:

- 写代码前先想归属:实现细节 → 开发文档;新踩到非显而易见坑 → CLAUDE.md 陷阱区(提炼浓缩,不重复开发文档);用户可感知 → README。
- **契约变更三件套**(代码层同步,与文档无关):`models.rs` + `src/types.ts` + `models.rs` 的 serde 回归测试必须一起改。
- CLAUDE.md 不展开开发文档已写清的东西,只写"为什么会坏"的约束。

## Commands

```bash
bun install                 # install deps
bun run tauri dev           # full dev app (Vite on fixed port 1420 + Rust compile)
bun run dev                 # frontend only
bun run build               # vue-tsc typecheck + vite build
bun run tauri build         # package desktop app
cd src-tauri && cargo test  # Rust unit + integration tests
```

单个 Rust 测试:`cargo test <name>`。`tests/commands.rs` 依赖 Cargo.toml 里显式的 `[[test]]` target(链接 comctl32 v6 清单)—— 不要把它挪进 `src/`。

## Non-negotiable invariants

这些一旦被改会静默出错,各有 serde 回归测试兜底:

- **IPC 全链路 camelCase。** `models.rs`、`src/types.ts`、serde 必须同步。
- **`ShellKind` 显式重命名为小写**(`powershell`/`cmd`/`bash`/`sh`)。serde 默认 camelCase 会产出 `powerShell`,破坏 `shells.json` 回读。
- **`OutputEvent` 需逐 variant 声明 `rename_all`**(枚举级只重命名 tag 不重命名字段)—— 否则 `runId` 以 `run_id` 传到前端,live 卡片挂掉。
- **`resolve_command` 用 args 数组构建进程,绝不字符串拼接** —— 命令/路径/cwd 走 argv 防注入与空格。别把它"简化"成插值。

## Frontend gotchas

- `start` 事件可能**先于 `invoke` 返回**到达 —— `store.handleOutput` 会先建占位 LiveRun,不要假设顺序。
- 每个 live run 内存只留最近 2000 行,磁盘日志完整。`runningCount`(侧边栏徽标)只统计 live 中 `running` 的任务。
- 页面用 `<KeepAlive>`,状态在 store 里 —— 已有数据别在页面 mount 时重复拉取。

## Editing

- 改 IPC:`models.rs` + `src/types.ts` + `models.rs` 里的 serde 回归测试一起改。
- Shell 管理约束:内置 Shell 不可删;被脚本引用的自定义 Shell 不可删;`save_shell` 按 exe 文件名推断 `kind`(前端不下发 `kind`)。
