# Lily-Toolbox

Windows 优先、本地优先的模块化个人生产力工作台：以一个入口管理桌面应用、Web Apps、文件和资料，优先保证稳定、快速启动、低内存与用户数据安全。

## 当前状态

WF-00 治理与工作流调整已经合并；WF-01 已建立最小 Tauri/React/Rust Shell 与 CI，并完成 Windows CI 及本机 Windows 11 运行复验，见[验收记录](docs/test-plan/WF-01-validation.md)。Shell 仅提供真实宿主健康检查，没有业务模块、设计系统或可发行安装/Portable 包。**未实现功能不作承诺**；检查状态见进度台账，源码或前端构建不等于 Windows 验收通过。

## V0.1 目标范围

- 桌面应用管理：发现、登记、搜索、收藏、启动、文件关联与基础状态。
- WebView2 浏览器/Web Apps：真实多标签、工作空间 Profile 隔离、收藏/历史/下载、Web App 登记与外部回退；宿主方案须先经 Windows SPIKE 验证。
- 文件管理、索引和基础转换：安全文件操作、文件名/元数据与 PDF/Markdown 全文搜索、PNG/JPEG/WebP、基本 PDF 和 ZIP 操作。
- 配套目标：SQLite 迁移、普通备份/恢复、基础权限、性能实测、Windows 安装版与目录化 Portable 发行。

AI、云同步、手机端、完整 Adobe 解析、第三方插件执行等按后续版本独立推进。当前不声明 Windows 11/Windows 10 运行兼容或性能达标。

## 文档入口

- [需求基线](TOOLBOX_SPEC.md)：V0.1 范围与验收唯一基线。
- [执行工作流](CODEX_WORKFLOW.md)：WF-00…WF-16、前置条件和 Gate。
- [开发规则](AGENTS.md)：每次任务必须遵循的约束。
- [进度与验证记录](docs/progress/CODEX_PROGRESS.md)：实际状态、证据和可复现检查。
- [产品待决事项](docs/progress/DECISIONS_PENDING.md)：需要所有者决策的 Gate 与范围变更。

## 开发与验证

版本与平台要求见[工具链说明](docs/architecture/toolchain.md)。Windows 本地先准备 Rust/MSVC/Windows SDK/WebView2；Node/pnpm 和 Rust 版本已固定，依赖由两份锁文件控制。

```powershell
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
pnpm desktop:dev
```

`pnpm dev` 仅启动前端服务，不能代替原生宿主；在普通浏览器中健康检查应显示失败。`pnpm desktop:dev` 打开真实桌面窗口。

```powershell
pnpm desktop:build
./tools/dev-scripts/windows-smoke.ps1 -Executable ./target/release/toolbox-desktop.exe
./tools/benchmarks/shell-baseline.ps1 -Executable ./target/release/toolbox-desktop.exe
```

WF-01 的 build 暂不生成安装包（`--no-bundle`）。冒烟测试观察真实主窗口、已渲染前端调用 Rust 的回执和正常关闭/重开；20 次重复启动工具记录 repeated-launch P90 和早期空闲完整进程树私有内存样本，不等于冷启动或 60 秒空闲 CPU 验收。CI 的原生构建、smoke 与报告实际完成后才可计入对应验证。生产界面无 Mock 成功，测试替身明确标记在测试文件中。

按用户授权顺序执行 WF-01 → WF-02 → WF-03，仍逐阶段验收与独立 PR；未通过 Gate 则先修复当前任务。WF-03.5 与后续功能没有在本次授权中执行。

需求映射：WF-00；ADR-002/011/012/014；TOOLBOX_SPEC §1、§11、§13–15。
