# ADR-017 — 浏览器宿主实验与待决方案

状态：proposed，Gate A 尚未通过，未获产品所有者批准。日期：2026-10-15。

关联：WEB-010 / WEB-007、ADR-003/006/014；WF-02。维持 Tauri 2 + React/TS + Rust 架构和所有 P0 基线，不提前实现浏览器业务或 UI。

需要选择一个真实单窗口多标签方案，并确认共享 Profile、Portable 与资源预算。实验比较：

1. Tauri unstable child WebView，高层管理器负责生命周期；独立 UDF 仅为对照。
2. Tauri Shell 中通过已有 Wry 的公开 Windows 扩展共享 Shell WebView2 环境并创建两个命名 Profile，所有控件留在窗口线程；高层 Tauri 管理器不跟踪这些原生子控件。
3. 如果 A 不可行，按工作流实际验证按需启动的独立 Browser Host，测量启动、进程协议、关闭和资源代价；不能只凭推测采用 B。

当前没有最终决定。方案 2 若实测可行，需要在生产浏览器适配层承担原生子控件的关闭、边界、焦点、DPI、异常和安全导航生命周期；实验成功不等于全部生产功能完成。直接复用已有 Wry/原生窗口句柄依赖，普通 Shell 不启用实验 feature。

证据：[WEB-01](../spikes/SPIKE-WEB-01.md)、[WEB-02](../spikes/SPIKE-WEB-02.md)、[PKG-01](../spikes/SPIKE-PKG-01.md)、[PERF-01](../spikes/SPIKE-PERF-01.md)。在实际 Windows 结果完整前，本 ADR 不标 accepted，WF-03 不启动。

回滚：撤销 WF-02 PR 的实验源码、feature 和 CI 步骤；不更改用户数据/Schema；保留 WF-01 已验收的普通 Shell。
