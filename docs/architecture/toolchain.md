# WF-01 工具链与运行边界

需求：APP-007（骨架部分）、ADR-003/011/012/014。仅建立最小 Shell，不宣称 APP-007 的单实例、配置或异常恢复已完成；这些仍按工作流后续落实。

- Node 24.19.0（`.node-version` / `engines`）；pnpm 11.25.0（`packageManager`）。
- Rust 1.99.0 / MSVC x64（`rust-toolchain.toml`）；Rust/TS 依赖由 Cargo.lock 和 pnpm-lock.yaml 固定。
- Tauri 2，React/TS/Vite；WF-01 **不启用** unstable 子 WebView，不确定最终多标签/Profile 方案。
- Windows 需要 C++ MSVC Build Tools、Windows SDK、WebView2 Evergreen。CI 使用 Windows x64 runner；本地缺少 MSVC 时不能把前端 build 当作原生编译成功。
- 只创建 core 与 desktop host 两个 Rust 包，以及一个最小共享 health DTO。无数据库/模块业务实现，无可选 Sidecar，无 FFmpeg/Adobe/AI 或自定义视觉系统。
- 前端依赖 React 与 Tauri API；测试/lint 工具只在开发阶段使用，不作为常驻服务。发布编译执行 tree shaking；记录实际 dist/exe 大小，不凭估算声称内存达标。
- 主窗口仅允许包内资源，开发模式例外仅限 127.0.0.1:1420；health/ready 命令通过 app manifest 与本地 main capability 最小授权，不给远程站点开放本地命令。
- `TOOLBOX_SMOKE_REPORT` 是测试启动进程指定的报告路径，前端不能传入文件路径。报告只包含 PID、版本、启动时间，不采集凭据或个人文件。

参考：[Tauri 先决条件](https://v2.tauri.app/start/prerequisites/)、[Tauri 配置](https://v2.tauri.app/reference/config/)。版本以实际锁文件为准，不在初始化时自动升级。
