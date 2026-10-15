# SPIKE-PKG-01 — Portable 路径

需求：SPIKE-PKG-01；规范 §6 / §12；WF-02。状态：in-progress；原生执行 NOT RUN。

本实验通过启动环境变量指定测试根目录，只作用于显式 `spike-web` feature，尚不是生产 Portable 标记/路径解析实现。先创建 `root/data` 并独占创建写探针，仅删除自己创建成功的探针；失败输出 `DATA_DIRECTORY_NOT_WRITABLE` 并退出非零，不回退到 AppData。

启动 A 路径，关闭重开 A，再启动包含空格和中文的 B 路径。验证实际 WebView2 Cookie：A 首次为空、重开持久、B 首次为空。检查共享 UDF 和命名 Profile 文件夹在对应 `root/data/webview/shell` 下，不发布 Cookie 数据库。

只读测试仅对本次新建目录设置当前用户的拒绝写入 ACL，必须非零退出并输出明确错误码；`finally` 恢复原 ACL。无递归删除用户路径。测试输出保留于 `.artifacts`，不提交实际 UDF、用户名、设备路径或敏感数据。

最低工作流验收：两个可写根目录及只读错误均需 Windows 实际证据。完整生产 Portable 分发、安装器、Runtime 缺失提示/安装、Windows 10、移动磁盘、并发实例/更新：NOT RUN，不能宣称已支持。

复现入口及命令见 [WEB-01](SPIKE-WEB-01.md)。测试夹具返回 PASS 仅说明测试逻辑，不能替代原生路径验收。
