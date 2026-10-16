# SPIKE-WEB-01 — 单窗口真实子 WebView

需求：WEB-010；WF-02。2026-10-15 Windows 11 实验 PASS；最终 Gate A 宿主确认待所有者审阅。

实验入口 `tools/spikes/windows-gate-a.ps1`，源码 `tools/spikes/wf02.rs`。仅 `spike-web` feature 编译，普通 Shell 不加载实验。受控 HTTP 夹具位于 `fixture-server.mjs`，不是模拟浏览器。

Tauri 2.12.3 `unstable` 子 WebView：单窗口先创建 10 个真实控件；随后 100 次操作分为切换、缩放、导航、关闭并重建各 25 次。每次检查管理器仍只有 11 个 WebView；完成真实下载并比对固定夹具字节后关闭全部子控件，检查管理器回到 1 个。只允许测试服务器同源导航和指定下载，不向远程页授予 native capability。

独立 UDF 是对照组；不能作为共享 UDF 命名 Profile 的通过证明。`profiles.rs` 另外用同窗口原生子控件验证命名 Profile 方案，详见 SPIKE-WEB-02。

复现：先执行前端构建，再 `cargo build --release --locked -p toolbox-desktop --features spike-web`；PowerShell 7 执行：

```powershell
./tools/spikes/windows-gate-a.ps1 -Executable ./target/release/toolbox-desktop.exe -OutputDirectory .artifacts/wf02-new-trial
```

输出目录必须不存在，避免将旧 Cookie 当成全新目录结果。原生报告和 HTTP 事件分别记录；只有二者均成功才视为执行通过。窗口创建/IPC/控件事件不能以 Node 单元测试代替。

实际结果：本地夹具 4 项 PASS；通用 CI 38720390893、Windows CI 38720390888 PASS。本机使用同一提交 d4ca638 的最终原生产物和脚本完整重跑，退出 0。Tauri 对照和共享环境 Wry 方案各创建 10 控件、完成 100 操作和实际下载，各有 60 个真实 HTTP 文档回报；切换/缩放/导航/关闭重建四种操作的实际覆盖数组均为 [10,10,10,10]。最终 Tauri 注册项回到 1，Wry 容器回到 0，宿主正常关闭后其进程树在 10 秒内退出。

这是有界生命周期检查，不是长期内存泄漏证明。长期泄漏、真实网站兼容性、输入法/无障碍/多屏 DPI：NOT RUN。完整证据与失败修复记录见 [WF-02 验收记录](../test-plan/WF-02-validation.md) 和 [原始聚合数据](../performance/wf02-local-baseline.json)。

资料：[Tauri 子 WebView API](https://v2.tauri.app/reference/javascript/api/namespacewebview/)。
