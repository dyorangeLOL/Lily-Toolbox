# WF-03 核心契约 v1

关联 ADR-009/010/011，规范 §6–8。WF-02 PR #4已合并；2026-10-16所有者确认ADR-017，Gate A pass。

原生入口 `core_dispatch({request})` 只向本地 main 窗口开放。请求固定为 `request_id/api_version/caller_context/params`，params为带`command/args`标签的类型化领域命令。来源取自宿主实际WebView；caller_context只是必须匹配toolbox.shell的声明，不能授予权限。拒绝未知字段、未知命令和非v1请求；输入上限64 KiB。

响应包含request_id/api_version和互斥的`success:true,data`或`success:false,error{code,message,retryable}`。文件路径、底层数据库错误和请求参数不进入错误或审计。request_id仅允许96字符内的ASCII标识。长任务以job_id和类型化事件契约表达；WF-03未启动长任务或伪造进度。未来订阅必须返回解绑函数并在页面卸载时执行。

七个领域已定义最小接口：apps.list、browser.listProfiles、files.list、search.query、convert.capabilities、backup.verify、modules.list/capabilities。此阶段仅后两项真实实现；文件类先检查授权范围，其余返回NOT_IMPLEMENTED。没有万能终端、进程启动或第三方代码执行入口。

共享数据位于packages/shared-types/fixtures/core.json，由Rust与TS双端解析/序列化测试。事件强制event_id/timestamp/source_module/schema_version及带标签payload；禁止额外敏感字段。尚未实现的模块生命周期是registered、features为空，不宣称可用。

数据路径由宿主解析，独立于工作目录：安装版用Tauri应用本地数据目录；Portable显式标记为根目录portable.flag，程序位于app目录，数据库为data/db/toolbox.sqlite；不可写直接失败，不回退AppData。该基础布局不是完整Portable发行验收。WebView目录与SQLite分离；浏览器数据迁移不在本任务。

数据库按需在阻塞工作线程首次调用时打开，主线程不做SQLite读写。连接开启WAL、foreign_keys和3秒busy_timeout。两次版本化SQL迁移用BEGIN IMMEDIATE统一事务执行；SHA256以LF规范化，跨平台换行不改变校验。拒绝修改后的迁移、断序、未来版本和坏库；不静默重建数据库。

最小Schema：schema_migrations、settings、modules、audit_events、permission_grants。仅核心持有Connection；未来模块用领域服务，不能直接写其他模块表。本次不创建应用/浏览器/文件/转换业务表。settings仅保存非敏感配置，不提供网页通用SQL或任意设置写入接口。

清单字段：id/version/api_version/kind/features/permissions/dependencies/entry/lifecycle。V0.1只接受原生注册的builtin入口；依赖清单预留但本版必须为空，不解析下载、不执行optional/sidecar或第三方入口。数据库清单也必须匹配编译时注册表，不能靠修改数据库增加权限。

文件授权仅原生入口可在用户明确同意后建立。默认拒绝、能力与模块匹配、目录作用域逐次核验，撤销实时生效；拒绝路径穿越、ADS、链接及Windows reparse point/Junction。当前只校验已存在路径，不执行文件动作；未来变更动作必须基于已打开句柄防止检查与使用间的路径替换，不能将本检查当作完整TOCTOU防护。

依赖：rusqlite 0.40.2（MIT，bundled SQLite，SQLite public domain）；sha2 0.10.9（MIT OR Apache-2.0）、semver 1.0.28（MIT OR Apache-2.0）复用锁定版本；tempfile仅测试。采用同步轻量SQLite加工作线程，不增加ORM、常驻数据库服务或Sidecar。包体/启动增量待实际产物测量，不能预先宣称无影响。参考：[rusqlite](https://docs.rs/rusqlite/0.40.2/rusqlite/)、[Tauri capabilities](https://v2.tauri.app/security/capabilities/)。

状态：in-progress。本机TS检查/测试已执行；Rust本机缺少MSVC link.exe，编译FAIL（环境）。Windows和Linux CI验证待运行，不计通过。WF-03.5保持not-started。
