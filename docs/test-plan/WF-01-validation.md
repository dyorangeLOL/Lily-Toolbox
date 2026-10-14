# WF-01 验证记录

日期：2026-10-14。需求：APP-007（最小 Shell 部分）、ADR-003/011/012/014。没有设计 UI 或实现后续功能。执行/自审：Codex；本次用户授权按 WF-01、WF-02、WF-03 顺序执行，逐阶段验收和独立 PR。

## 真实检查

| 检查 | 环境 | 结果 | 证据 |
|---|---|---|---|
| 冻结安装、peer、TS strict、lint、format、前端构建 | 本地 Node 24.19 / pnpm 11.25 | PASS | pnpm install --frozen-lockfile；peers check；typecheck；lint；format:check；build |
| TS DTO 与界面错误/协议/就绪测试 | 本地 / Ubuntu CI / Windows CI | PASS | pnpm test：10 项通过；Mock 只在测试文件 |
| Rust fmt | 本地隔离 Rust 1.99 | PASS | cargo fmt --all -- --check |
| Rust core clippy 与共享 DTO/error 测试 | Ubuntu CI | PASS | [Common checks 第三轮](https://github.com/dyorangeLOL/Lily-Toolbox/actions/runs/38584099171)，3 项 Rust 测试 |
| Windows workspace clippy/test 与原生 release 编译 | Windows Server 2025 runner | PASS | [Windows 第二轮](https://github.com/dyorangeLOL/Lily-Toolbox/actions/runs/38583195220)；完整执行成功 |
| 真实 WebView2 渲染后 IPC、原生窗口、正常关闭重开 | Windows runner / 本机 Windows 11 | PASS | windows-smoke.ps1：2 次启动通过；不是 Mock 前端 |
| 20 次重复启动 | Windows runner / 本机 Windows 11 | PASS（采样完成） | shell-baseline.ps1；不等于 cold boot 或性能全项达标 |
| 本地 Rust 编译 | 本机 Windows 11 | FAIL（环境） | 实际错误 link.exe not found；本机没有 MSVC，采用已验证 CI 产物做运行复验 |
| 第一轮 Windows 编译 | Windows runner | FAIL → 已修复 | [第一轮](https://github.com/dyorangeLOL/Lily-Toolbox/actions/runs/38582369588) 缺少 icon.ico；补官方模板及 MIT 许可后复测通过 |

TypeScript 7 与 eslint 解析器不兼容已改为受支持 TS 6.0.3；异步绘制回执时序测试已修复。没有隐藏失败或将跳过的步骤计为通过。

## 本机采样与来源

- Windows 11 家庭版 x64，build 26200；Intel Core Ultra 9 185H（16 核 / 22 逻辑处理器），约 32 GiB RAM；WebView2 155.0.4283.45。
- 原生二进制来源 commit `06407bebeb3bf53d6073a5268b6424cfec088052`，GitHub artifact `11904841201`；采样脚本来自 `d792ea56d3e8b0f1bd17ebae0b72f72a9f4b84b3`。两提交的原生/前端运行代码相同，后者仅改采样与文档。
- ZIP SHA-256 `a9c01f4aea1ca240e800676a7421d0067c5bf10176eed23cff73b2bb2f8cccc3`，下载后实际比对通过。
- EXE 大小 8,304,128 bytes；SHA-256 `5EC55A6B0BE357CC2F0613DF113E7AC225BB937D84446B254069659BAD55FCE9`。
- 20 次 repeated-launch：launcher P90 **905.7557 ms**，Rust 入口至渲染后前端回执 P90 **799 ms**；早期空闲完整进程树私有内存最大 **220,987,392 bytes**（约 210.75 MiB）。数值并非未来多标签或任意机器的保证。
- [本机采样数据](../performance/wf01-local-baseline.json) 与[来源/硬件](../performance/wf01-provenance.json)是脚本实际输出的脱敏数据；不包含设备路径、用户名、Cookie 或账号信息。

## 复现与未运行项

在配齐 Windows Rust/MSVC/SDK/WebView2 的环境冻结安装后运行 README 中的检查与 `pnpm desktop:build`。本机无 MSVC 时可取得对应 CI artifact，先核对 digest 再运行：

```powershell
./tools/dev-scripts/windows-smoke.ps1 -Executable ./target/release/toolbox-desktop.exe
./tools/benchmarks/shell-baseline.ps1 -Executable ./target/release/toolbox-desktop.exe
```

这两项会启动真实原生窗口；正常关闭后重开，报告仅记录本次进程的渲染/IPC回执、窗口与退出、启动时间、早期空闲进程树私有内存。临时诊断由启动进程提供路径，不能从网页指定。

冷/热启动定义与完整资源冻结在 WF-02：使用隔离测试数据目录，分别采集 20 次独立冷启动和 20 次同目录重开；说明操作系统缓存条件、硬件/系统/版本及计时起点，禁止混用 repeated-launch 结果。补齐 60 秒空闲 CPU、1/5 标签与索引/转换场景的进程树测量；不得只统计 Shell，也不自动重启用户机器或删除现有个人数据。

**NOT RUN**：实际手动鼠标/键盘与屏幕阅读器验收；冷启动定义/预算冻结、60 秒 CPU、多标签/Profile、Portable、Windows 10 兼容、安装包。对应后续阶段才能计入验收。当前真实可运行的是最小 Shell，不是 Preview 产品发布。

Code Review（Codex 自审）：运行代码仅健康/就绪；命令由 app manifest 与 local main capability 限制；正常生产不写测试报告；远程导航不开放；无重型模块启动链；版本/锁文件和错误分支有真实测试。没有独立审查人参与，不冒充所有者对未来 Gate 的批准。回滚独立 PR #3，不变更用户原件和规范。
