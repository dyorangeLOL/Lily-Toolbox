# Lily-Toolbox

Windows 优先、本地优先的模块化个人生产力工作台：以一个入口管理桌面应用、Web Apps、文件和资料，优先保证稳定、快速启动、低内存与用户数据安全。

## 当前状态

仓库处于 WF-00 治理与规则建立阶段，仅有需求、工作流和开发文档。尚无应用源码、可运行程序、CI、安装包或 Portable 包。**未实现功能不作承诺**；需求和路线图不代表已完成能力。

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

一次只执行一个已指定任务。WF-00 验收、审查与合并后，才单独派发 WF-01；工具链、开发启动命令和 CI 在 WF-01 实际建立，当前不提供不存在的构建命令。

需求映射：WF-00；ADR-002/011/012/014；TOOLBOX_SPEC §1、§11、§13–15。
