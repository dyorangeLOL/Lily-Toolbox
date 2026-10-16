# WF-02 实际验收记录

日期：2026-10-15。执行/自审：Codex。需求：WEB-010、WEB-007、SPIKE-PKG-01/PERF-01；独立 [PR #4](https://github.com/dyorangeLOL/Lily-Toolbox/pull/4)。实验时Gate A等待所有者审阅；2026-10-16所有者明确确认ADR-017方案，Gate A pass。该实验未实现WF-03、WF-03.5或后续功能。

## 实际执行

本机 Windows 11、WebView2 Evergreen 155.0.4283.45；硬件/原始数据见 [基线 JSON](../performance/wf02-local-baseline.json)。源码与产物摘要见 [provenance](../performance/wf02-provenance.json)。ZIP SHA256 核对后才解压执行，不发布签名下载 URL 或真实 UDF。

```powershell
node --test tools/spikes/fixture-server.test.mjs
pnpm typecheck
pnpm lint
pnpm test
pnpm build
cargo fmt --all -- --check
./tools/spikes/windows-gate-a.ps1 -Executable <verified-artifact>/target/release/toolbox-desktop.exe -OutputDirectory <new-output-directory>
```

最后一条实际命令使用本机 `.artifacts/wf02-d4ca638/target/release/toolbox-desktop.exe` 和全新 `.artifacts/wf02-local-full-02`，没有选择 SkipPerformance，执行退出码 0。尖括号仅是公开复现路径占位，不声称直接运行了占位命令。

| 检查 | 环境 | 结果 |
|---|---|---|
| Node HTTP/下载、拒绝错误隔离/越权、重开、同 Profile 共享验证器 | 本机；通用CI | PASS，4项 |
| TS 类型/lint/10项测试/build、Rust格式、PowerShell解析 | 本机 | PASS |
| Rust clippy 本机 | Windows / 无MSVC | FAIL（环境）：link.exe not found；不计通过 |
| 普通及 spike-web feature clippy、workspace test、两种 release 编译 | Windows CI 38720390888（提交d4ca638） | PASS；3项Rust测试 |
| Tauri 10控件/100操作/60真实文档/下载/回收 | 本机 Windows 11 | PASS |
| 共享同一环境、两命名Profile、10控件/100操作/60文档/下载/回收 | 本机 Windows 11 | PASS |
| 同域隔离、同Profile共享、远程native拒绝 | 两原生方案 | PASS；没有授予远程capability |
| Portable A初次/重开、B中文空格路径、只读ACL | 本机 Windows 11 | PASS；拒写退出2，ACL恢复 |
| 空Runtime目录的真实SDK可用性拒绝 | 本机 Windows 11 / 最终CI | PASS（预期失败）；WEBVIEW_RUNTIME_UNAVAILABLE，退出2，不进入框架初始化等待；生产友好安装提示尚未实现 |
| 正常退出后的拥有进程树回收 | 本机所有启动试验 | PASS，10秒内无存活成员 |
| 20次process-cold / 20次warm、0/1/5页60秒进程树 | 本机 Windows 11 | 采样PASS；最终cold/warm P90=0.907/0.791秒，所述口径预算PASS；初轮warm 2.159秒FAIL完整保留 |
| 真正OS缓存冷启动、长期泄漏、Win10、安装器、签名、真实账号/DRM网站、生产Portable | 未执行 | NOT RUN，本阶段不承诺 |

## 已发生的失败与修复

1. CI 38678935134：远程锁文件内容截断；改为逐文件 blob 上传并核对本地 Git SHA，修复后解析/冻结构建通过。
2. CI 38679471619：直接 cargo 实验产物没有 custom-protocol，实际主页面未加载；feature 加入自定义协议并先等待前端真实就绪，后续复测通过。
3. CI 38714550789：真实控件/Cookie均通过，随后磁盘目录断言错误；实测名称是 WV2Profile_workspace_a/b，修正后本机完整重跑通过。没有放宽Cookie或环境隔离要求。

4. CI 38716267807：Runtime空目录的错误路径未能在20秒内退出，不能计为通过；在实验feature的主窗口构建失败分支先写诊断并明确退出2，避免进入Tauri setup panic/系统崩溃对话框。e5a2e32复测仍失败，进一步定位见下一项。

5. CI 38717671666：仅在构建返回错误后退出仍不足以解决等待，缺少 Runtime 报告表明阻塞发生在错误返回之前；改用官方 GetAvailableCoreWebView2BrowserVersionString，明确传入同一进程的 Runtime 目录覆写，先验证真实可用性再进入窗口初始化。b2a3a69的Windows CI 38719825793随后PASS，最终d4ca638再次通过CI和本机完整流程。

6. 自审修正操作轮转的覆盖缺口：原轮转会使切换/关闭各落在固定一半标签。最终d4ca638按操作独立轮转，两个方案都记录[10,10,10,10]真实覆盖；修正后整套流程重新运行，没有用旧记录冒充新覆盖。

## 审查与限制

Codex 自审已对照 WF-02 四项实验、权限、数据目录、取消/退出、进程树、计时与未执行项。所有原生浏览器数据都是受控夹具；实际远程站点、第三方插件执行、SQLite和业务UI未实现。源码只在显式 feature 编译，普通 Shell 启动链不载入实验。

100次操作只证明该有界试验未发现控件注册项增长和退出残留，不证明无限时长内存无泄漏。Runtime缺失测试通过只表示预期失败被捕获并有诊断，不表示友好安装流程已完成。最终版本冷/热启动复测达标；保留初轮热启动超标158.7ms的数据，不把两轮差异归因单一改动；需要保持同一口径回归。

最终宿主建议见 [ADR-017](../adr/ADR-017-browser-host.md)。Codex未代替所有者批准Gate A；2026-10-16所有者明确确认方案并授权继续WF-03。原始JSON的待审状态保留为实验时快照。



证据一致性检查实际 PASS：40个样本的nearest-rank P90与摘要一致、最终热启动PASS且初轮FAIL保留、两组HTTP/原生计数一致、进程私有内存求和一致、60秒CPU/PID稳定、负面路径结果、ZIP与EXE SHA256一致；公开JSON不含用户路径、密钥或真实Cookie。

最终代码检查：[通用CI](https://github.com/dyorangeLOL/Lily-Toolbox/actions/runs/38720390893) 与 [Windows CI](https://github.com/dyorangeLOL/Lily-Toolbox/actions/runs/38720390888) 全部PASS；Windows CI显式SkipPerformance，完整40次启动及3组60秒CPU只引用本机实测。最终EXE与脚本均来自d4ca638，哈希见provenance。5页单浏览器进程另做同产物专项快照，观察12个拥有进程、1个browser主进程，Cookie/IPC验证和正常退出PASS。
