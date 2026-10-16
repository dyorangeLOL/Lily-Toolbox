# SPIKE-WEB-02 — 共享环境与命名 Profile

需求：WEB-007、ADR-006；WF-02。2026-10-15 本机 Windows 11 共享环境/Cookie 实验 PASS；宿主 ADR 等待所有者确认。

Tauri 2.12.3 的高层 `WebviewBuilder` 没有命名 Profile 配置。其当前锁定的 Wry 0.57.0 提供 `WebViewBuilderExtWindows::with_profile_name` 和 `with_environment`。本实验直接复用已有 Wry，不修补第三方源码、不引入第二套渲染引擎。

`tools/spikes/profiles.rs` 在 Tauri 主窗口线程获取 main 的 `PlatformWebview.environment()`；每个 Wry 子控件明确使用同一 COM 环境并指定 `Workspace_A` 或 `Workspace_B`。UDF 是主 Shell 的 `data/webview/shell`。COM 和控件保留在线程局部容器，工作线程只发送操作和接收结果；HWND 为主窗口生命周期内的借用。原生子控件不在 Tauri WebView 管理器中，分别记录 `attached_webviews`、`native_children`，不能混称。

同一 localhost 域的页面仅在不存在 Cookie 时写入 Profile 标记；验证 A/B 不串用、同 Profile 的多个控件共享数据、同 Portable 路径重开保留 Cookie、另一根目录初次启动为空。页面向夹具服务器报告真实 `document.cookie` 和视口；尝试 native health 命令必须拒绝或不存在 IPC bridge。

同时执行 10 控件/100 次生命周期及下载；完成后清空控件容器。Portable 检查必须在共享 UDF 下找到两个真实命名 Profile 文件夹。性能采样使用该方案的 0/1/5 标签完整进程树，不能拿独立 UDF 的数据宣称共享环境成本。

许可/依赖：Wry 0.57.0（MIT OR Apache-2.0）、raw-window-handle 0.6.2（MIT OR Apache-2.0）以及 webview2-com 0.39.1（MIT，用于真实Runtime预检）均已存在于 Tauri 依赖树；新增只是 Windows 实验 feature 的直接引用，锁文件无新增包。最终实验产物体积、内存和启动已记录于 PERF-01；未测同提交启用/禁用 feature 的增量，不能把与 WF-01 的差额归因于单个依赖。

实际结果：两命名 Profile 同域隔离、同 Profile 多控件共享、重开保留、另一 Portable 根目录首次为空均 PASS；10 控件/100 操作、下载和关闭均 PASS。5 标签活跃时的独立进程快照观察到完整进程树 12 个进程，但只有 1 个 WebView2 browser 主进程；未发布命令行。资源明细见 [PERF-01](SPIKE-PERF-01.md)。Cookie 结论不涵盖尚未实现的历史/收藏夹 SQLite 隔离。

风险：高层 Tauri 未管理原生子控件；后续生产适配必须明确关闭/resize/DPI/焦点/进程异常处理，不能直接将实验当成浏览器模块。建议同窗口 Wry 适配方案，最终决定见 [ADR-017](../adr/ADR-017-browser-host.md)。A 路线已有可实施证据，因此未触发独立 Browser Host B；B 未运行，不宣称通过。

参考：[Microsoft 多 Profile](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/multi-profile-support)、[Wry Windows 扩展源码](https://github.com/tauri-apps/wry/blob/wry-v0.57.0/src/lib.rs)。接口存在不是 Windows 实测通过证明。

实测补充：当前 Runtime 将逻辑 Profile Workspace_A/B 映射为 UDF 下 EBWebView/WV2Profile_workspace_a/b；测试按实测映射检查，不能假设目录名等于逻辑名称。生产实现应通过 WebView2 Profile API 查询路径，不把此磁盘布局当成稳定契约。
