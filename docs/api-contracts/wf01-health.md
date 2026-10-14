# WF-01 最小健康契约

需求：ADR-011/012；完整领域协议由 WF-03 定义，本文件不提前实现。

本地 `main` 前端调用 `health_check`，响应严格包含 `api_version: 1`、`version: SemVer`、`runtime: "tauri-rust"`。TS 校验与 Rust 反序列化共用 `packages/shared-types/fixtures/health.json` 测试样例，拒绝不支持版本/额外字段。生产响应来自实际 Rust 宿主，浏览器预览无法产生 native 成功。

前端健康检查通过并渲染后调用 `frontend_ready`，正常运行没有文件写入；测试进程提供 TOOLBOX_SMOKE_REPORT 时才记录本地启动诊断。诊断写入错误返回 `{code, message, retryable}`，不得输出设备路径。

验证层级：单元测试用明确测试替身验证错误/协议处理；Windows 冒烟运行真实 WebView2 前端和 Rust 命令并观察窗口与正常退出。单元测试不计为 Windows 验收。
