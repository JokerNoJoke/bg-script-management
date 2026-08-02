# 脚本管理后台

本地桌面工具:把常用的命令行任务(部署、清理、构建、巡检等)固化成脚本,一键运行并实时查看输出。单机、单用户、无需登录,脚本以当前用户权限运行。

## 功能

- **脚本库** — 维护可复用脚本(命令或脚本文件),支持搜索、增删改;也可临时粘贴一段命令直接运行,不保存进库
- **运行控制台** — 实时流式查看运行输出(错误标红),随时终止任务、可设超时自动结束
- **运行历史** — 自动记录每次运行(保留最近 500 条),可筛选、回看完整日志、一键重跑
- **Shell 管理** — 自动识别本机 Shell(PowerShell、CMD、Git Bash 等),也可以添加自定义

## 安装运行

基于 [Tauri 2](https://tauri.app/),桌面应用。构建需要 Node.js、[bun](https://bun.sh/) 和 Rust 工具链。

```bash
bun install         # 安装依赖
bun run tauri dev   # 开发模式运行
bun run tauri build # 打包应用(产物在 src-tauri/target/release/)
```

## 发布新版本

GitHub Actions 自动构建并发布 Windows 与 macOS(Apple Silicon / Intel)安装包到 [Releases](https://github.com/JokerNoJoke/bg-script-management/releases)。无需本地打包。

1. 更新版本号:改 `src-tauri/tauri.conf.json` 的 `version`
2. 推送代码后打标签:
   ```bash
   git tag v1.0.0   # 版本号与 tauri.conf.json 保持一致
   git push origin main --tags
   ```
3. 在 Actions 页查看构建进度,完成后安装包出现在 Releases 页

> 直接推 `main`(不打 tag)也会触发构建,并使用当前版本号自动创建对应 Release(同版本更新同一 Release)。构建产物为未签名,Windows 首次运行会有 SmartScreen 提示。

## 文档

- **开发者** 请阅读 [DEVELOPMENT.md](DEVELOPMENT.md)(架构、数据模型、运行流程、测试)。
