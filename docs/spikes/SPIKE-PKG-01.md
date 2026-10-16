# SPIKE-PKG-01 — Portable 路径

需求：SPIKE-PKG-01；规范 §6 / §12；WF-02。2026-10-15 本机 Windows 11 路径与错误实验 PASS。

本实验通过启动环境变量指定测试根目录，只作用于显式 `spike-web` feature，尚不是生产 Portable 标记/路径解析实现。先创建 `root/data` 并独占创建写探针，仅删除自己创建成功的探针；失败输出 `DATA_DIRECTORY_NOT_WRITABLE` 并退出非零，不回退到 AppData。

启动 A 路径，关闭重开 A，再启动包含空格和中文的 B 路径。验证实际 WebView2 Cookie：A 首次为空、重开持久、B 首次为空。检查共享 UDF 和命名 Profile 文件夹在对应 `root/data/webview/shell` 下，不发布 Cookie 数据库。

只读测试仅对本次新建目录设置当前用户的拒绝写入 ACL，必须非零退出并输出明确错误码；`finally` 恢复原 ACL。无递归删除用户路径。测试输出保留于 `.artifacts`，不提交实际 UDF、用户名、设备路径或敏感数据。

最终结果：两个可写根目录、A 重开持久性、B 中文/空格路径初次隔离 PASS；只读 ACL 触发 DATA_DIRECTORY_NOT_WRITABLE、退出 2，ACL 已恢复。通过进程局部 WEBVIEW2_BROWSER_EXECUTABLE_FOLDER 指向空夹具目录，官方 GetAvailableCoreWebView2BrowserVersionString 明确检测不可用，输出 WEBVIEW_RUNTIME_UNAVAILABLE、退出 2，系统 Runtime 未修改。最终 CI 与本机都通过，未伪造 SDK 结果。

初轮产物在本机走到 setup panic（退出101），CI 则在错误返回前的初始化等待中阻塞。最终预检显式使用同一个 Runtime 目录覆写，在进入框架初始化前拒绝此错误条件；保留后续构建失败的非零退出兜底。这里只交付测试与诊断，不把 JSON 诊断称为生产级原生安装提示。SDK 参考：[Microsoft WebView2 Globals](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl?view=webview2-1.0.3912.50)。

完整生产 Portable 分发、安装器、Runtime 缺失的原生友好提示/安装、Windows 10、移动磁盘、并发实例/更新：NOT RUN，不能宣称已支持。基线只验证浏览器数据，SQLite/应用设置尚未实现。

复现入口及命令见 [WEB-01](SPIKE-WEB-01.md)。测试夹具返回 PASS 仅说明测试逻辑，不能替代原生路径验收。
