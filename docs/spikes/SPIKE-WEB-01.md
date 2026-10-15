# SPIKE-WEB-01 — 单窗口真实子 WebView

需求：WEB-010；WF-02。状态：in-progress，Windows 实验待执行，Gate A 未通过。

实验入口 `tools/spikes/windows-gate-a.ps1`，源码 `tools/spikes/wf02.rs`。仅 `spike-web` feature 编译，普通 Shell 不加载实验。受控 HTTP 夹具位于 `fixture-server.mjs`，不是模拟浏览器。

Tauri 2.12.3 `unstable` 子 WebView：单窗口先创建 10 个真实控件；随后 100 次操作分为切换、缩放、导航、关闭并重建各 25 次。每次检查管理器仍只有 11 个 WebView；完成真实下载并比对固定夹具字节后关闭全部子控件，检查管理器回到 1 个。只允许测试服务器同源导航和指定下载，不向远程页授予 native capability。

独立 UDF 是对照组；不能作为共享 UDF 命名 Profile 的通过证明。`profiles.rs` 另外用同窗口原生子控件验证命名 Profile 方案，详见 SPIKE-WEB-02。

复现：先执行前端构建，再 `cargo build --release --locked -p toolbox-desktop --features spike-web`；PowerShell 7 执行：

```powershell
./tools/spikes/windows-gate-a.ps1 -Executable ./target/release/toolbox-desktop.exe -OutputDirectory .artifacts/wf02-new-trial
```

输出目录必须不存在，避免将旧 Cookie 当成全新目录结果。原生报告和 HTTP 事件分别记录；只有二者均成功才视为执行通过。窗口创建/IPC/控件事件不能以 Node 单元测试代替。

已执行：夹具 HTTP 与验证器测试本地 4 项 PASS；首次草稿通用 CI 38678690052 PASS（当时 3 项夹具测试）。Windows 原生构建、100 次操作、下载、控件回收：NOT RUN，等待 PR #4 的 Windows 构建产物。连续长时间内存泄漏、真实网站兼容性、输入法/无障碍/多屏 DPI：NOT RUN，100 次实验不能证明这些能力。

资料：[Tauri 子 WebView API](https://v2.tauri.app/reference/javascript/api/namespacewebview/)。
