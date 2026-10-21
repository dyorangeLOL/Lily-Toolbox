# WF-03 验收记录

日期：2026-10-21。执行/自审：CoCo（Codex）。关联 ADR-009/010/011、规范 §6–8；[PR #5](https://github.com/dyorangeLOL/Lily-Toolbox/pull/5)。本记录只覆盖核心协议、数据库与权限基础；WF-03.5 未开始。

## 范围与代码审查

- [核心契约](../api-contracts/wf03-core.md)、[ADR-018](../adr/ADR-018-core-foundation.md)：七个领域的类型化边界，只有模块清单/能力查询真实可用，保留未实现错误；不提供通用执行、SQL、第三方入口。
- 主窗口本地 capability、实际来源与 caller_context 分离，参数64 KiB上限，拒绝未知字段/版本。按需工作线程迁移，原生状态跨await使用自有引用，不扩大远程权限。
- SQLite两次事务迁移、SHA256换行规范化、WAL、外键、参数化查询；数据库损坏/未来版本/迁移校验错误都拒绝，不覆盖或重建。
- 写探针仅删除自己成功创建的文件；只读失败不回退；文件作用域和实时撤销、链接/Junction拒绝；目前不执行文件动作，不宣称句柄级防竞态已完成。
- 授权/撤销和审计同事务；日志不含请求参数、原生错误、完整路径、Token或Cookie。测试及artifact只含拥有的夹具与脱敏报告。

## 实际检查

| 检查 | 结果与证据 |
|---|---|
| 本机 pnpm typecheck / pnpm test / pnpm format:check | PASS；共享类型34项、最小Shell3项，共37项 |
| 本机 cargo fmt --all -- --check | PASS |
| 本机 Rust 编译 | FAIL（环境）：缺少MSVC link.exe；未计通过 |
| 通用CI e700c6d / 528a4fb | PASS：类型/lint/格式/37项TS测试/构建、4项HTTP夹具、Rust clippy和核心测试 |
| 最终 Windows 构建、单元测试、原生探针与真实IPC | 待当前产物，NOT RUN；不能用Linux结果替代 |
| 本机下载验证后的 Windows 原生实测 | NOT RUN，等待产物 |
| UI设计、实际主题、屏幕阅读器、Windows10、长期稳定性、安装包 | NOT RUN，本阶段未实施或不在本任务范围 |

## 已发生的失败

1. 38877294583：迁移checksum函数被同名变量遮蔽，E0618；已修复。
2. 38877573483 / 38877573624：AppError缺少Display导致共享契约反例测试不能编译；已增加仅输出错误码的Display/Error，通用CI通过。
3. 39597055070：异步Tauri命令持有借用State参数，宏要求Result且生命周期不满足；改为等待前从AppHandle克隆自有状态，保持响应契约；等待Windows复测。

待Windows证据完整且代码审查通过后再标verified。工作流§22的所有者审阅不由自审代替；后续只有前置验收满足才开始WF-03.5。
