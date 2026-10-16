# ADR-017 — 浏览器宿主实验与待决方案

状态：accepted；Gate A pass。产品所有者于2026-10-16明确确认方案并授权继续WF-03；实验日期2026-10-15。

关联：WEB-010 / WEB-007、ADR-003/006/014；WF-02。维持 Tauri 2 + React/TS + Rust 架构和所有 P0 基线，不提前实现浏览器业务或 UI。

需要选择一个真实单窗口多标签方案，并确认共享 Profile、Portable 与资源预算。实验比较：

1. Tauri unstable child WebView，高层管理器负责生命周期；独立 UDF 仅为对照。
2. Tauri Shell 中通过已有 Wry 的公开 Windows 扩展共享 Shell WebView2 环境并创建两个命名 Profile，所有控件留在窗口线程；高层 Tauri 管理器不跟踪这些原生子控件。
3. 如果 A 不可行，按工作流实际验证按需启动的独立 Browser Host，测量启动、进程协议、关闭和资源代价；不能只凭推测采用 B。

建议方案：保留 Tauri Shell，在同一个原生窗口内通过 Wry 公开扩展创建共享环境的命名 Profile 子控件（方案 2）。实际 WEB-A/WEB-B/PKG-A 已通过 Windows 11 实验；5页进程树仅一个浏览器主进程，Shell-only 约190 MB、空闲CPU约0.016%。最终版本冷/热各20次 launcher P90 为0.907/0.791秒，满足所述口径目标；初轮热启动2.159秒的失败记录保留，不对速度差异作无证据归因，**不降低性能要求**。所有者已依据工作流 §22 明确确认采用此方案，保持现有性能目标，Gate A 记 pass。

这是对高层 Tauri child 管理方式的具体调整：Tauri 继续管理 Shell/窗口，Wry 适配层直接拥有浏览器子控件；不能称为高层 Tauri 已原生支持命名 Profile。生产浏览器适配层必须承担关闭、边界、焦点、DPI、异常和安全导航生命周期；实验成功不等于全部生产功能完成。直接复用已有 Wry/原生窗口句柄依赖，普通 Shell 不启用实验 feature。A 的技术路线可实施，未触发独立进程 Host B；未运行 B，不宣称 B 通过。

证据：[WEB-01](../spikes/SPIKE-WEB-01.md)、[WEB-02](../spikes/SPIKE-WEB-02.md)、[PKG-01](../spikes/SPIKE-PKG-01.md)、[PERF-01](../spikes/SPIKE-PERF-01.md)、[验收记录](../test-plan/WF-02-validation.md)。自审由 Codex 完成，不能冒充产品所有者确认。所有者现已确认 Gate A，WF-02 PR合并后按原顺序进入 WF-03；WF-03.5 未执行。

回滚：撤销 WF-02 PR 的实验源码、feature 和 CI 步骤；不更改用户数据/Schema；保留 WF-01 已验收的普通 Shell。
