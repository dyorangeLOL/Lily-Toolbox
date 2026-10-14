# Toolbox V0.1 — Codex Cloud 分步执行工作流

> 文档版本：1.1｜日期：2026-10-14｜状态：可执行任务编排（不是代码完成证明）
> 需求唯一基线：仓库根目录 `TOOLBOX_SPEC.md`（2026-10-14 v1.0）  
> 目标：完成可安装、可便携运行的 Toolbox V0.1 Windows x64 Preview。  
> **UI/UX 设计集中在 WF-03.5，必须在 WF-02、WF-03 验收通过后单独执行；此前只允许真实操作与测试必需的最简交互，此后按已验收设计逐项实现。**

本次 v1.1 调整仅新增设计工作包与相应依赖，不改变 `TOOLBOX_SPEC.md` 的功能需求、技术架构、权限边界或性能目标。若现有 `AGENTS.md` 仍写有“一律不设计视觉 UI”，项目所有者本次授权仅对 **WF-03.5 设计阶段及后续对应任务采用其已验收规范**构成有限例外；其他任务仍不得擅自设计或重做 UI。本次只修改本文，不同步改写规则、进度台账或实际 UI，也不执行 WF-03.5。

---

## 0. 如何使用（项目所有者先读）

1. 在 GitHub 创建或准备 `Toolbox` 仓库，将现有 `TOOLBOX_SPEC.md` 与本文 `CODEX_WORKFLOW.md` 放在仓库根目录并提交。**Codex Cloud 只能可靠读取已提交到它所连接仓库的文件**。
2. 在 Codex Cloud 连接此仓库，选择正确分支；先发送下面的**首次启动提示词**。
3. 每个 `WF-XX`（包括 `WF-03.5`）是**独立、可验收的任务**。一次只派发一个任务；审核生成的 diff、测试证据和 PR，合并后再派发下一项。文档中的阶段顺序不是要求 Codex 在一个会话里连续执行全部任务。
4. 某一步依赖 Windows 11 实际桌面/WebView2/打包验证时，Codex Cloud 必须提交 Windows CI 或本地执行步骤，状态停留在 `awaiting-windows-validation`，**不得**声称通过。
5. 你现在有 Windows 11 + VS Code；只需要负责执行不能在云端验证的 Windows 步骤、检查结果，并审批涉及架构更改、发布或风险的关卡。
6. 本文所有命令以最终仓库实际的包管理器和目录为准。若真实脚本名不同，Codex 必须使用仓库中实际存在的命令，不得伪造成功日志。
7. 不提供任何外部账户凭据、Adobe 授权材料或现有个人 Web App 私有 URL 到公开仓库；此类数据保存在本地配置或 CI Secrets。

### 0.1 首次启动提示词（复制给 Codex Cloud）

```text
你是 Toolbox 仓库的实现代理。请先读取根目录 TOOLBOX_SPEC.md 与 CODEX_WORKFLOW.md，严格执行其中的边界和验收标准。

本次只执行 CODEX_WORKFLOW.md 的 WF-00；不要提前实现 WF-01 或后续任务，不设计 UI，不重写产品范围。

交付本任务要求的源码/文档、真实可执行的验证命令与测试结果，更新 docs/progress/CODEX_PROGRESS.md 和对应的验收状态；创建或提交一个独立 PR（如果当前环境支持），在 PR 描述中列出需求 ID、改动、验证、Windows 未验证项、风险与回滚方案。

如果依赖/环境阻塞，保留已验证的成果，写明阻塞原因和下一步，不用 Mock 结果冒充 Windows 实测。不要因为技术阻碍悄悄删除 P0 要求。不要一次性执行整份工作流。
```

### 0.2 下一任务提示词（每次只替换任务号）

```text
阅读 TOOLBOX_SPEC.md、CODEX_WORKFLOW.md、根目录 AGENTS.md（若存在）和 docs/progress/CODEX_PROGRESS.md。
只执行 CODEX_WORKFLOW.md 中的 WF-XX（到设计阶段时填 WF-03.5），前置条件未满足就报告阻塞，不跨越阶段 Gate。

优先复用仓库已有代码和接口；只修改本任务必需文件。实现真实功能、编写测试并执行能在当前环境运行的检查。对 Windows 特定事项提供 CI 证据或本地复现说明，未执行的测试标记 NOT RUN。

完成后更新进度与文档，产出独立可审查的 diff/PR，最终回答按「实现 / 文件 / 测试（PASS/FAIL/NOT RUN）/ 风险 / 需要 Windows 验证 / 下一任务」汇报。
```

### 0.3 Windows 验证完成后的继续提示词

```text
以下是 WF-XX 在 Windows 11 的实机测试结果和日志：（粘贴结果或上传日志）
请仅针对本任务修复失败项，记录真实复测证据并更新 docs/progress/CODEX_PROGRESS.md。
不得绕过当前 Gate 进入下一功能任务；若涉及技术路线调整，先提交 docs/adr/ 中的方案和影响说明。
```

---

## 1. 绝对执行规则

### 1.1 需求优先级及变更权限

- `TOOLBOX_SPEC.md` 为 V0.1 范围的唯一正式基线；本文规定实施顺序。冲突时优先维持基线 P0 与安全要求，记录冲突，**不得自行删除或放宽**。
- V0.1 三个必做核心：**桌面软件管理、WebView2 浏览器/Web Apps、文件管理/索引/基础转换**；附带 SQLite、普通备份、基础安全和 Windows 发布。
- V0.1 真实可用：**禁止仅有按钮、假的列表/登录/下载/转换、无意义的 Promise resolve、硬编码演示成功**。Mock 仅可用于明确命名的单元测试夹具。
- 不自行扩展到独立 PS/AE/PR 编辑器、完整 Adobe 工程解析、AI API/本地模型、云同步、手机端、系统级自动化平台、第三方插件运行、浏览器扩展市场。
- 文件删除默认进回收站；转换默认另存；普通远程网页**没有** Rust/Tauri 特权调用能力。
- 不能把全部模块和大型媒体引擎都打包到单个 EXE。采用轻量 Shell、懒加载模块以及经验证才启用的按需 Worker/Sidecar。
- 避免新增依赖；若增加二进制依赖，记录版本、许可、体积、启动影响、替代方案。

### 1.2 每个任务的固定交付协议

Codex 必须在改动前确认本任务涉及的需求 ID；每任务交付：

1. 可审查的最小改动与必要测试（避免一个 PR 横跨无关模块）。
2. 更新 `docs/progress/CODEX_PROGRESS.md` 中的当前任务状态、测试与实际证据链接。针对本次“仅修改工作流”的授权，不改台账；下一次获准维护台账时在 WF-03 与 WF-04 之间补入 WF-03.5，并以 `not-started` 初始化，不因规划已写入而标为完成。
3. 新增或更新相应的 `docs/adr/ADR-*.md`、`docs/api-contracts/`、迁移文档（仅在架构/契约变化时）。
4. 在 PR 描述列出：需求 ID、变更内容、测试 PASS/FAIL/NOT RUN、Windows 待验证、权限/数据影响、启动/内存影响、已知限制、回滚方式。
5. 因环境限制不能执行的命令必须标 `NOT RUN` + 原因，不得虚构已运行。

建议任务状态：`not-started / in-progress / awaiting-windows-validation / verified / blocked / deferred`。只有相关测试证据完整、Code Review 通过，才标 `verified`。

### 1.3 统一开发和测试惯例

- 前端：TypeScript `strict`；React + Vite；统一 DTO 校验；测试使用 Vitest。WF-03.5 前只提供可操作的最简工作界面；WF-03.5 制定 UI/UX 规范，后续功能任务按规范实现本任务所需界面，不另起无关视觉重设计。
- 桌面：Tauri 2 + Rust；优先 Windows WebView2 Evergreen；只有验证后才决定是否启用 Tauri `unstable` 子 WebView API。
- 存储：SQLite + WAL + 事务 + 外键 + 版本化迁移；FTS5；不得在主线程处理大型文件扫描和转换。
- 建议锁定 `pnpm`/Node LTS 与 Rust stable 版本，并提交对应 lockfile；具体版本按创建仓库时稳定可用的受支持版本选定，记录在 `docs/architecture/toolchain.md`。
- 每个实际实现任务运行可执行的检查，例如：`pnpm install --frozen-lockfile`、`pnpm -r typecheck`、`pnpm -r test`、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`。**只有仓库已定义这些脚本/工作区时才照此执行**。
- Windows 专属检查使用 `.github/workflows/windows-build.yml`、Windows Runner 或本地 PowerShell；不得用 Linux 的 `cargo check` 代替 Windows 功能验收。
- 性能目标见 `TOOLBOX_SPEC.md §9`：冷启动 P90 3 秒、热启动 P90 2 秒、无标签 Shell 目标 ≤300 MB 私有内存、空闲 CPU <2%，**均为实测目标而非凭空保证**。冻结基线需实测 20 次并记录机器配置。
- 不将个人 URL、访问 Token、浏览器 Cookie、SSH Key、设备路径或个人数据提交到仓库。

### 1.4 Gate 和依赖图

```text
WF-00 规则与进度基线
   ↓
WF-01 最小可运行桌面骨架 + CI
   ↓
WF-02 Windows 架构验证（WEB / PROFILE / PORTABLE / PERF）<--- GATE A
   ↓
WF-03 核心领域协议 + SQLite 迁移 + 能力白名单
   ↓（WF-02 verified / GATE A pass + WF-03 verified）
WF-03.5 UI/UX Design System（设计交付验收，不实施业务 UI）
   ├── WF-04 / WF-05 应用发现与启动
   ├── WF-06 / WF-07 / WF-08 浏览器与 Web Apps
   └── WF-09 / WF-10 / WF-11 文件管理、搜索与转换
            ↓
WF-12 外部软件和现有 Web Apps 集成
   ↓
WF-13 普通备份/恢复 + 中断恢复
   ↓
WF-14 安全、性能与稳定性回归 <--- GATE B
   ↓
WF-15 安装版/便携版与 Windows CI <--- GATE C
   ↓
WF-16 Preview 发布审查与 GitHub Release <--- GATE D
```

**串行默认**：保持 WF-00 → WF-01 → WF-02 → WF-03 → WF-03.5 → WF-04 的顺序，其他原有任务编号与相对顺序不变。仅在 WF-03 的命令/事件契约已冻结、WF-03.5 已验收、分支互不冲突且有独立 PR 时，WF-04/05 与 WF-09/10 等任务才适合并行。浏览器仍必须通过 GATE A 后再实现。WF-03.5 不替代或放宽 GATE A/B/C/D。

---

## 2. WF-00 — 仓库治理与 Codex 执行约束

**目标**：建立 Codex 能在每次任务中复用的最小规则与状态机制；不做业务功能。  
**依赖**：`TOOLBOX_SPEC.md`、`CODEX_WORKFLOW.md` 已在仓库根目录。  
**需求**：ADR-002/011/012/014；规范 §13–15。

**实施**：

1. 检查仓库是否已有 `AGENTS.md`、代码、CI、lockfile、README；不得覆盖现有有效内容。
2. 创建简洁根目录 `AGENTS.md`（不超过约 120 行），规定：需求优先级、一次一个任务、禁止伪造验证、权限默认拒绝、不得在启动链加载大型模块、修改文件须保留原件、PR 报告格式；指向本文和对应 `TOOLBOX_SPEC.md` 章节。
3. 创建 `docs/progress/CODEX_PROGRESS.md`，列出 WF-00…WF-16（包括 WF-03.5）状态、需求映射、PR/commit、验证人、云端测试、Windows 测试、阻塞和最新更新。
4. 创建 `docs/progress/DECISIONS_PENDING.md`，只收录需产品所有者决定的 Gate 或变更；不要把普通工程实现细节推回用户。
5. 在 README 中加入项目使命、V0.1 范围、文档入口与“未实现功能不作承诺”。
6. **不要**一次性生成大量空 crate、空目录或占位模块。

**验收**：新增文档路径存在、互相引用正确、不会将后续任务提前标为完成；若仓库已有内容，记录审查结论。  
**任务输出**：`AGENTS.md`、`docs/progress/CODEX_PROGRESS.md`、`docs/progress/DECISIONS_PENDING.md`、README 最小变更。  
**完成标志**：`WF-00 verified`，才派发 WF-01。

---

## 3. WF-01 — 工具链、最小 Shell、可复现构建与 CI

**目标**：运行一个真实 Tauri 2 + React + TypeScript + Rust 桌面程序，并建立持续检查。**不设计视觉 UI**。  
**需求**：APP-007、ADR-003、ADR-011/012。

**实施**：

1. 检查现有项目，如果是空仓库，则建立 `apps/desktop` 最小 Tauri 2 + Vite + React TS 结构；最简控件可展示 Tauri/Rust 健康检查结果，不做页面设计稿。
2. 创建尽量少的 Rust crate：先有 `toolbox-core`（错误模型、基本类型）和 host；其余按实际需要建立。
3. 设置 lint、format、TS strict、lockfile、Rust 工具链说明、统一错误结构；提供 `README` 本地开发与运行步骤。
4. 配置 `.github/workflows/ci.yml` 运行 TS/Rust 通用测试；配置 `.github/workflows/windows-build.yml` 验证 Windows x64 编译及最小启动/打包可行性（若尚不能可靠冒烟，明确标注并留下手动步骤）。
5. Windows 构建依赖（MSVC 工具链、WebView2 等）应记录在文档；不要假定 Linux Codex Cloud 能启动 Windows GUI。
6. 测量最小 Shell 的冷/热启动与内存（如 Windows CI 不适合做固定性能基准，则提供本地采集工具和步骤）。

**云端检查**：实际定义的前端类型检查、单元测试、Rust 格式、lint、test。  
**Windows 检查**：能构建、启动和关闭；记录运行截图或日志、系统版本、commit SHA。  
**验收**：真正启动到可交互窗口；CI 真实执行并有结果；未做业务功能；没有未经必要的大型常驻依赖。

**Gate**：未在 Windows 11 实际启动之前，不能把“桌面程序运行成功”标为已验证。

---

## 4. WF-02 — 四项优先技术 SPIKE 与 GATE A

**目标**：决定是否可以继续依赖 Tauri 宿主完成 P0 浏览器与 Portable；**本任务主要是实验和 ADR，非大规模业务开发**。  
**需求**：WEB-010、WEB-007、SPIKE-PKG-01、SPIKE-PERF-01；规范 §12。

### 4.1 必须完成的实验

| 编号 | 实验 | 最低可重现证据 | 决策 |
|---|---|---|---|
| WEB-A | Windows Tauri 子 WebView | 创建 10 个真实网页视图，累计 100 次打开/切换/关闭/缩放，检查稳定性与泄漏 | `pass/fail/needs-review` |
| WEB-B | Profile 隔离 | 共享 UDF + 两个 Profile 对同一测试域验证 Cookie 隔离；记录是否真的共享 WebView2 环境 | `pass/fail` |
| PKG-A | Portable 数据路径 | 两个不同可写路径重新启动，确认数据目录隔离；只读路径明确报错 | `pass/fail` |
| PERF-A | Shell/WebView2 资源 | 20 次冷/热启动；Shell-only/1 tab/5 tabs 的进程树内存、CPU 基线 | 实测预算与优化建议 |

优先使用受控 `localhost` 测试页，不使用需要账号或 DRM 的站点做浏览器基础 Gate。

### 4.2 浏览器回退分支

- **A 方案 PASS**：记录所用 Tauri 版本、`unstable` feature、权限和稳定性结果；使用 Tauri child WebView 作为正式架构。
- **A 方案 FAIL**：先验证保持 Tauri Shell 并按需启动的独立 Windows WebView2 Browser Host 是否能真实创建和控制多标签、保留 Profile 隔离，测量额外启动和内存成本。
- **B 方案 PASS**：提交 ADR，说明子进程协议、进程退出、WebView2 数据路径、授权和资源预算；通过评审后采用 B。
- **B 方案也 FAIL**：停止 WEB 业务编码，在 `DECISIONS_PENDING.md` 记录更换宿主框架的方案与代价，等待产品所有者确定；**不可以采用截图/iframe 假装有 Edge 多标签**。

Tauri 官方目前将同一窗口创建子 WebView 标为 `unstable`，并需要相应 Cargo Feature；稳定的多 `WebviewWindow` 并不自动等于用户要求的单窗口多标签。参考：https://v2.tauri.app/reference/javascript/api/namespacewebview/

### 4.3 本任务产出

- `docs/spikes/SPIKE-WEB-01.md`
- `docs/spikes/SPIKE-WEB-02.md`
- `docs/spikes/SPIKE-PKG-01.md`
- `docs/spikes/SPIKE-PERF-01.md`
- `docs/adr/ADR-0XX-browser-host.md`（编号由实际仓库决定）
- 可重现实验源码（测试夹具/临时原型可保留在 `tools/`，不进入生产启动链）
- Windows 构建产物/测试说明、视频或文字操作日志、资源采样文件（注意脱敏）。

**GATE A 通过条件**：实际 Windows 证据足以选择**一个**可实施的多标签方案，Profile/Portable 可行，性能基线已记录；未经验证不能开始 WF-06。

---

## 5. WF-03 — 核心领域、数据层与受控命令 API

**目标**：先确定跨模块通信与数据库基础，避免三个模块各自发明不兼容的状态结构。  
**需求**：ADR-009/010/011；规范 §6–8。

**实施**：

1. 定义稳定的 `request_id / api_version / caller_context / params` 命令包和 `success / data / error { code, message, retryable }` 响应；长任务返回 `job_id`，用类型化事件报告进度。
2. Rust 领域接口只做必要集合：`apps`、`browser`、`files`、`search`、`convert`、`backup`、`modules`。TS DTO 与 Rust 序列化有 Contract 测试；不提供 `executeShell` 等万能命令。
3. 建立 SQLite 数据路径解析（安装版/AppData、Portable/data），自动迁移、WAL、外键、参数化查询、迁移失败回滚；建立最少必要 schema。
4. 设计一份短的模块清单 Schema（`id/version/kind/features/permissions/dependencies`），**V0.1 不执行第三方插件**。
5. 添加最小权限检查/审计入口：默认拒绝远程网页对宿主命令的直接访问，并校验文件作用域。
6. 实现有意义的错误码及基础结构化日志；日志不得输出敏感字段。

**测试**：迁移/重开、并发写入、坏数据、版本升级、DTO 双端验证、远程页越权拒绝、Portable 路径不可写失败。  
**验收**：各模块可通过统一接口实现功能，不需直接写入其他模块内部表；数据库升级正常；没有任何外部网页获得通用本地执行权限。

---

## 5.5. WF-03.5 — UI/UX Design System

**目标**：在已验证宿主与核心契约基础上，建立适用于后续三个核心模块的统一 UI/UX 设计规范；定位为**现代、简洁、专业、紧凑的 Windows 桌面生产力软件**。本阶段交付设计文档与设计资产，不实现业务页面，不重构实际 UI。
**需求映射**：ADR-003/005/010/011/014；规范 §2、§3、§7–9、§11、§14–15。覆盖已有 P0 的呈现与操作，不新增功能需求；P1/P2 仅保留扩展规则，不提前实现。
**强制前置**：WF-02 `verified` 且 GATE A `pass`（含 Windows 实测和已批准的宿主 ADR），以及 WF-03 `verified`（命令/事件、模块清单、数据与权限契约已通过验收）。两者任一未通过时不执行设计任务，保持 `not-started` 或记录 `blocked`；不能拿临时 UI 掩盖宿主或契约问题。

### 5.5.1 任务边界与设计依据

- 保持 Tauri 2 / React / TypeScript / Vite / Rust / SQLite、已验收浏览器宿主和模块边界；设计层通过已有 DTO/命令/事件工作，不重新定义领域模型、数据库 Schema、宿主权限或插件运行时。
- 参考 Windows 11 的桌面层级与键盘操作、GitHub 的清晰信息组织与状态反馈、VS Code 的紧凑工作区与扩展入口理念；记录参考来源和采用理由，不能照搬品牌或为仿外观改变产品需求。
- 优先复用仓库已采用的成熟设计体系和组件库；基于规范 §3.1 的 Tailwind CSS、Radix/shadcn 与 Lucide 储备评估，必要时比较 Fluent 等成熟体系的 React 适配。执行此阶段时核对官方文档、版本与许可，说明可访问性、维护、按需导入、包体与性能代价，再选择一套一致方案；不并存多套大型组件体系，不为视觉另换技术栈。
- 紧凑不等于微小文字或难以点击：定义密度、可读性、键盘焦点、DPI 缩放与窗口缩放规则；以桌面鼠标/键盘高频操作为主。所有样例须标“设计稿 / 非功能验证”，不展示伪造的实时操作结果。

### 5.5.2 详细任务

1. **信息架构**：列出工作台、应用管理、浏览器/Web Apps、文件/搜索/转换、任务反馈、设置/备份等已有能力的层级、入口、上下文和主流程；将每个入口映射到规范需求 ID 与后续 WF。定义全局导航、模块内导航、工作空间/Profile、内容区与任务状态的职责，避免同一能力重复散落；不新增独立业务模块。
2. **设计系统**：定义语义色彩、字体与字阶、间距、尺寸/密度、边框、圆角、层级、焦点、状态色和轻量动效 tokens。采用统一语义名称，不在页面硬编码颜色或样式。记录正常、hover、focus、active、selected、disabled、loading、empty、error 等状态；无障碍规则包括键盘操作、可见焦点、文本对比度与非纯颜色状态提示。
3. **页面规划**：交付 Shell 及三核心模块、设置/普通备份的页面矩阵和线框；每页说明目标、需求 ID、入口/退出、主要动作、数据来源、空态/加载/错误/权限拒绝/取消状态、后续实施 WF。浏览器线框服从 WF-02 已批准的真实 WebView 边界；不将复杂 DOM 覆盖层或假标签方案当作已可用能力。长列表/大目录规划增量加载或经验证的虚拟化，页面未激活时不预加载全量数据。
4. **交互规范**：规定鼠标、键盘导航、焦点转移/恢复、菜单、对话框、表单校验、搜索反馈、进度、取消/重试、错误恢复与外部打开回退。快捷键不能冲突或全局夺取系统按键；不提前实现全局快捷键等 P1。危险操作确认、覆盖冲突、回收站删除和转换另存遵循原安全需求；不添加未经规范授权的“撤销成功”承诺。动效仅用于反馈或状态衔接，定义 reduced-motion 与无动效降级。
5. **主题系统**：完整定义 Light、Dark 的等价语义 tokens 与组件状态；规划系统主题跟随、用户显式选择、初始主题读取和避免闪烁的规则。未来自定义主题只预留版本化 token 格式、允许字段、校验、对比度检查和缺失/无效值回退；不引入任意脚本/CSS 执行或远程主题加载，不在此阶段实现主题市场。使用 CSS variables 等与现有栈兼容的轻量方案，不为切换主题重建 WebView、重载业务数据或整页重复初始化。
6. **组件规范**：列出导航项、工具栏、按钮、输入/搜索、列表/表格、标签页、菜单、对话框、状态/进度、错误/空态等可复用组件；逐项说明职责、输入/事件、状态、键盘/ARIA、Light/Dark、密度、数据量边界及实施归属。优先成熟可访问性基础组件，业务封装保持薄层；组件不得绕过已有命令或权限契约。只定义规范和接口示例，不修改生产组件。
7. **图标规则**：仅使用 Lucide 等正规图标库，确定一套主库及尺寸、描边、对齐、语义命名与可访问名称。禁止 Emoji、Unicode 符号或临时手写 SVG 充当界面图标；此限制不禁止图标库自带 SVG 渲染。采用显式按需导入及可控图标 ID 映射，缺失图标回退到主库的通用图标或文字，不加载整个图标集合或执行模块提供的任意 SVG/脚本。
8. **动态模块导航**：规划基于 WF-03 模块清单/能力状态和前端注册适配层生成入口，核心 Shell 只保留全局容器与必要入口，禁止枚举所有未来功能的固定导航数组。说明稳定 module ID、标题、分组/排序、路由、图标 ID、可见性、能力/权限与加载状态的映射；显示元数据缺口先记录映射/适配方案，不能擅自扩改已验收的领域契约。设计新增/启用/禁用/卸载/失败时的入口更新、当前页面失效回退、路由冲突和未知图标处理；以标注清楚的设计场景验证新增模块无需修改 Shell 导航。**这里只规定未来动态安装模块的 UI 兼容性，V0.1 仍仅内置模块，不开发安装器、市场、第三方代码执行或热卸载保证。**
9. **性能与稳定性约束**：复用 WF-02 基线和规范 §9，不调整指标。规划路由/模块/组件按需加载、受控列表更新、监听解绑和主题切换成本；不得预载全部页面、图标、字体或模块。禁止为视觉使用大面积实时模糊、持续粒子/背景动画、重型动效引擎、额外渲染 WebView 或导致明显负担的装饰；优先系统字体、简单 CSS 与静态层级。列出组件方案的依赖成本依据、风险和后续测量方法；未知数据明确标待测，不能以估算宣称低内存或启动达标。

### 5.5.3 交付物（执行 WF-03.5 时才创建）

| 路径 | 内容与可审查证据 |
|---|---|
| `docs/design/README.md` | 设计入口、风格定位、成熟体系/组件库评估与选择理由、官方参考、需求映射及实施归属 |
| `docs/design/information-architecture.md` | 信息架构、导航/工作空间职责、主流程、动态模块注册适配与生命周期场景 |
| `docs/design/design-system.md` | tokens 规则、排版/密度/层级、颜色与图标、无障碍和轻量动效约束 |
| `docs/design/pages.md` | 已有 P0 页面矩阵、带需求 ID 的线框及全部关键状态；设计资产链接/版本，不制造虚假功能证据 |
| `docs/design/interactions.md` | 输入/焦点/快捷键、菜单/对话框、异步任务、错误/权限、安全操作及回退流程 |
| `docs/design/themes.md` 与 `docs/design/tokens.json` | Light/Dark 语义 token 对照与机器可读设计数据；未来自定义主题格式、校验和回退规则 |
| `docs/design/components.md` | 组件契约/状态矩阵、可访问性、主题/密度、按需使用与实施责任，不含生产组件实现 |
| `docs/design/validation.md` | 前置 Gate 证据、设计审查/对比度/token/映射检查、性能风险和后续 Windows 验证计划；逐项 PASS/FAIL/NOT RUN |

按任务协议更新进度台账并提交独立 PR，记录审查人、设计资产版本和与 WF-04…WF-16 的映射；如需改变架构或核心契约，停止该变更并另行提出 ADR，本阶段不能以设计决策替代架构验收。不得在本阶段安装运行依赖、修改 `apps/desktop`、新增业务页面或实现未来模块。

### 5.5.4 检查、验收与退出条件

- **前置证据**：WF-02 与 WF-03 的验收/PR/commit/Windows 证据可追溯，GATE A 已通过；缺证据不进入阶段。
- **完整性与范围**：六类设计内容齐全；每个页面/主流程/组件均可追溯到既有需求和实施任务；所有交付路径/链接可访问，无新增 P0、技术栈/宿主/数据层/权限变更。文档/JSON 使用当前实际可运行的检查工具，记录真实命令与结果。
- **主题与可访问性**：Light/Dark 覆盖同一套语义 tokens、关键组件与状态；无颜色字面量散落的设计约定；自定义主题缺失/无效值有回退；检查文本对比度、键盘顺序、可见焦点与非纯颜色提示。设计审查和工具结果留档，不冒充实际 Windows 屏幕阅读器测试。
- **图标与扩展**：组件/线框图标可映射到正规库，未用 Emoji/Unicode/临时手写 SVG；动态模块新增、移除当前页、禁用/失败、未知图标与路由冲突均有场景审查，新增模块不要求改 Shell 导航；未来安装能力未被标为已实现。
- **安全与资源**：危险操作/权限拒绝/错误恢复沿用原约束；组件体系无不必要重叠；懒加载、列表数据量、监听生命周期、轻量主题与动效规则明确；没有为视觉引入明显性能负担的方案。性能风险审查通过不等于已达成运行指标。
- **Windows 验证**：本阶段不修改实际 UI，应用主题切换、DPI/缩放、真实键盘/屏幕阅读器、动态导航运行及资源测量均为 `NOT RUN`（原因：尚未实施）；列明后续各功能任务的真实 Windows 验证步骤，在 WF-14 做完整回归。不能为取得 `verified` 把设计稿当作运行测试。
- **退出**：所有设计交付、可执行文档/数据检查与设计审查通过，并记录实际证据后，才标 WF-03.5 `verified`；其 PR 审查/合并后单独派发 WF-04。设计未通过则修复同一任务，不开展后续业务 UI。此处验收的是设计规范，不是运行时 UI 或 V0.1 产品完成。

---

## 6. WF-04 — 应用发现、登记与持久化

**目标**：实现真正可用的软件启动器的数据基础。  
**需求**：APP-001/002/003（部分）。

**前置**：WF-03.5 `verified` 且设计 PR 已审查/合并，WF-03 原有契约仍有效；仅采用设计规范实现本任务的真实应用管理界面，不提前开发其他页面。

**实施**：

1. 在 Windows 下扫描合理的安装来源（开始菜单 `.lnk`、已安装程序登记信息等），过滤无效条目并限制耗时，不强制管理员权限。
2. 提供手动添加 EXE/.lnk 的能力，保存名称、目标、参数数组、图标引用、分组等；支持修改、移除、自定义与扫描数据合并。
3. 提供关键词搜索、收藏、最近启动历史等持久化业务逻辑；在最简交互界面中能实际调用。
4. 程序图标加载可异步处理，失败时有降级，不应拖慢首屏。

**测试**：真实 Windows 安装程序扫描、无效快捷方式、重名、特殊字符路径、重启后持久化、重复扫描不产生无限重复记录。  
**验收**：能发现至少 1 个真实已安装程序，手动添加 1 个 EXE，搜索/收藏并重开后保留。

---

## 7. WF-05 — 软件启动、文件关联、状态与固定命令

**目标**：完成 V0.1 应用管理 P0。  
**需求**：APP-004/005/006/007、INT-001。

**实施**：

1. 使用 Windows 支持的结构化启动接口，不将用户输入拼接成 `cmd.exe` 或 PowerShell 字符串。
2. 实现启动 EXE/快捷方式、系统默认文件关联打开、显式指定软件打开、受控参数数组；验证路径、存在性和特殊字符处理。
3. 检测应用运行状态（最佳努力）；无法检测时为 `unknown` 而非虚假的 `not running`。
4. 固定白名单快捷命令、单实例处理、异常重开基础恢复；错误需可解释。

**测试**：安装路径含空格、引号、Unicode、无权限/已移动程序、已运行进程、指定程序打开真实文件、双击启动两次。  
**验收**：能启动已安装程序、指定程序打开测试文件、显示真实/未知状态；没有命令注入路径。

---

## 8. WF-06 — 真正的 WebView2 多标签运行时

**前置 Gate**：WF-02 GATE A `pass` + 浏览器宿主 ADR 已批准 + WF-03.5 `verified` 且设计 PR 已审查/合并；保留 WF-03 和原有浏览器任务依赖。
**目标**：真实 WebView2 标签的打开、导航、切换、关闭、恢复。  
**需求**：WEB-001/002/008/009。

**实施**：

1. 通过已验证的宿主实现网页生命周期，至少支持 HTTPS、用户授权的 localhost，以及未来可信打包 Web App 的明确策略。
2. 标签生命周期：`created/loading/committed/completed/failed`；能打开、切换、关闭、调整大小；崩溃时错误可见而不是伪造成功。
3. 防止非 HTTPS 外部跳转、`file://`、自定义 scheme 等绕过权限边界；支持系统浏览器回退。
4. 初步会话恢复；5 个真实网页标签反复切换，单个网页失败不摧毁主程序。
5. 禁止为方便开发而将 `window.__TAURI__` 或敏感宿主 API 暴露给任意远程网站。

**测试**：localhost/公开 HTTPS、无效 URL、导航失败、100 次标签操作、窗口缩放、多窗口切换、外部跳转；Windows 真实 WebView2 进程观察。  
**验收**：真实页面承载且 100 次操作无主程序崩溃；不存在 iframe 假替代。

---

## 9. WF-07 — 工作空间、Profile 隔离与浏览数据

**目标**：浏览器的独立工作空间和基础持久化。  
**需求**：WEB-003/004/007、规范 §4。

**实施**：

1. 建立工作空间、默认 Profile 和 Web App 覆盖 Profile 的引用关系；`profile_id` 使用稳定内部 ID。
2. WebView2 Cookie/站点存储与 Toolbox 的书签/历史记录按 Profile 隔离；尽可能复用一个 UDF 下的多个 Profile，实际能力以 Windows 测试为准。
3. 收藏夹新增、编辑、删除；历史存储、按 Profile 查询、搜索和清理；重开后保留。
4. 清理 Profile 前关闭其相关 WebView，清理行为不得影响其他 Profile。

**测试**：两个 Profile 访问同一测试站点设置不同 Cookie；退出重启后数据隔离；一个 Profile 清理历史不影响另一 Profile。  
**验收**：真实隔离、真实持久化；不能只靠 SQL `profile_id` 假装 Cookie 已隔离。

---

## 10. WF-08 — 下载、Web Apps 与外部回退

**目标**：完成基本浏览器/Web App 体验，并集成现有记账/摄影应用的注册能力。  
**需求**：WEB-005/006/008、INT-004、WEB-011（另列可选验证）。

**实施**：

1. 记录下载的来源、目标路径、大小、进度、成功/失败/取消；选择保存位置，确保危险路径和重名处理。
2. Web App 注册：名字、URL、工作空间、Profile 覆盖、首选打开方式（内嵌/系统浏览器/原生 App）；测试重启后恢复。
3. 内置 Web App 模板仅为**不包含私有 URL 的用户可填写配置**。已有记账和摄影工作台不提供真实 URL 时，不应写死示例私有地址。
4. 若第三方网站拒绝嵌入、OAuth 登录或媒体播放，提供清晰的外部浏览器/官方客户端回退。
5. 增加 `SPIKE-WEB-04-apple-music.md` 的独立验证任务：验证公开 Apple Music Web 界面可否在 WebView2 正常登录和播放；没有 Apple Developer Program，不做 MusicKit 自制播放器。此 SPIKE 失败**不阻断**本任务 P0。

**测试**：真实文件下载、失败/取消、非法保存路径、Web App 多次重开与 Profile 配置、回退打开流程。  
**验收**：可以登记真实网址、持续打开和下载；Apple Music 仅报告实测状态，不承诺播放成功。

---

## 11. WF-09 — 安全文件浏览与基本操作

**目标**：先完成真正的文件管理，后续全文索引与转换才有可靠基础。  
**需求**：FILE-001/002、INT-005、ADR-008。

**前置**：WF-03 + WF-03.5 `verified` 且设计 PR 已审查/合并；文件业务契约与原件保护规则不变。

**实施**：

1. 目录浏览、排序、文件详情、分页/增量读取；大目录时界面不能卡死。
2. 复制、移动、重命名、新建文件夹；删除进入 Windows 回收站；重名时有跳过/改名/经确认的覆盖策略。
3. 长文件操作异步执行，有明确进度、取消、错误报告，批量变更提供预览。
4. 文件到桌面应用操作通过受控 API；向普通网页不得默默传递真实本地路径。

**测试**：多层目录、无权限路径、Unicode/特殊字符、长路径、符号链接/Junction、跨磁盘移动、覆盖冲突、中途取消、回收站。  
**验收**：所有基础操作真实完成，原文件不会因转换/取消被误删；回收站操作可复测。

---

## 12. WF-10 — 后台索引、SQLite FTS5、PDF/Markdown 全文搜索

**目标**：兑现第二轮确认的“首版必须有 PDF/Markdown 全文搜索”。  
**需求**：FILE-003/004/005/006。

**实施**：

1. 首次运行告知默认索引范围（桌面、文档、下载、图片、视频），允许排除目录和加入自定义目录；禁止无条件全盘扫描。
2. 分两个阶段索引：文件元数据 → Markdown 与**可提取文本** PDF；SQLite FTS5 存储搜索内容索引。
3. 索引任务运行在后台，支持取消、暂停、增量更新、关闭后恢复；必要时再决定单独 `indexer` Sidecar，不能先启动空闲 Sidecar。
4. 索引大小限制、忽略规则、损坏文件降级与性能日志；扫描件 PDF OCR 不属于 V0.1。
5. 文本提取库需评估许可与异常文件安全；为基本 FTS 查询提供稳定的 API。

**测试**：至少 1000 个文件的测试目录，Markdown、文本 PDF、损坏 PDF、删除/重命名文件、重复扫描、暂停恢复、权限拒绝。  
**验收**：文件名/属性查询与两个格式全文查询都命中正确，操作过程不中断 UI，源文件未修改。

---

## 13. WF-11 — 轻量格式转换：图片、PDF、ZIP

**目标**：V0.1 基础转换真实可执行，但不把 FFmpeg/Adobe/Office 等大型依赖塞进 Shell。  
**需求**：FILE-007/008/009；规范 §5。

**实施**：

1. 先验证稳定库与许可并记录 `docs/spikes/SPIKE-FILE-01.md`；引入最小转换能力查询接口。
2. 图片：PNG/JPEG/WebP 相互转换，测试尺寸、方向、透明通道、错误输入；HEIC 等其他格式为后续模块。
3. PDF：基本合并、拆分、图片转 PDF；使用经过验证的成熟库，异常或加密 PDF 返回明确错误。不可提取 OCR 不阻断。
4. ZIP：创建、列出与解压；严格拒绝 Zip Slip、异常符号链接、超限解压、压缩炸弹；不得将输出写到指定目录外。
5. 每个任务异步、可取消、有进度或明确阶段状态；原文件不动，输出默认另存；任务结束释放内存和临时文件。
6. 大型未来转换器只有 Capability/Manifest 协议，不开发虚假的“已安装扩展包”入口。

**测试**：正确文件比对、错误输入、相同文件名、不可写目录、磁盘空间不足（可模拟受控错误）、取消、中断、ZIP 路径穿越和大小限制。  
**验收**：每一类真实输出均可被独立程序打开；没有悄悄覆盖输入，也没有大型空闲转换进程。

---

## 14. WF-12 — 外部软件与 Web App 的统一登记

**目标**：把 Adobe、开发、办公、AI 网站及现有两套 Web App 作为**受控集成入口**，不侵入它们的私有文件结构。  
**需求**：INT-001…005、INT-007（独立研究）。

**实施**：

1. 检测 Photoshop、Premiere Pro、After Effects 是否已安装；使用 Windows 支持的应用入口启动；未安装时显示明确状态。
2. 正确识别 `.psd/.psb`、`.prproj`、`.aep/.aepx`，通过系统关联/指定软件打开；可对工程目录建立**文件索引**，不修改工程内容。
3. ChatGPT/Claude/Gemini、Notion、GitHub 及用户的记账/摄影 Web Apps：使用统一 Web App 注册机制，用户自行填写地址；不在公开配置保存个人密钥。
4. 通过规范化的“从文件定位到打开应用/复制安全信息”接口联动；远程 Web App 暂不获得原生读取权限。
5. `SPIKE-ADOBE-01` 可独立研究（使用非生产示例工程），输出可行性与受支持的官方 API 路线，但不进入 V0.1 工程解析交付。

**测试**：未安装/已安装应用、工程关联、文件路径特殊字符、Web App 地址校验、重启后配置恢复。  
**验收**：不需要安装 Adobe 就能安全运行 Toolbox；能够登记用户真实的 Web Apps；不声称会修改 Adobe 内部图层/时间线。

---

## 15. WF-13 — 普通备份/恢复、数据库迁移与异常中断

**目标**：用户数据能够安全恢复，Portable 程序升级不覆盖个人数据。  
**需求**：规范 §6、AC-011。

**实施**：

1. 实现 SQLite 一致性快照以及 settings、工作空间、书签、应用列表等**普通备份**；不默认包含浏览器 Cookie、密钥或用户的全部原始文件。
2. 恢复先校验备份格式、版本与完整性，在隔离临时目录验证，再由用户确认应用；恢复失败不得毁坏当前库。
3. 实现 Schema migration 升级、失败回滚和重启恢复；打开程序时不要执行耗时扫描。
4. Portable 升级测试保留 `data/`；Program Files/只读目录清晰失败/明确授权用户更换数据位置。
5. `Encrypted Backup` 的 API 可以预留，但不在 V0.1 声称已实现安全加密备份。

**测试**：正常备份/恢复、坏备份、旧 schema 升级、断电/强制退出模拟、恢复取消、敏感字段缺失验证。  
**验收**：数据可恢复，正常备份不含账号凭据，恢复操作不擅自覆盖用户当前数据。

---

## 16. WF-14 — 安全、稳定性与性能回归（GATE B）

**目标**：完成发布前横向审查，而不是仅靠每个单功能的 happy path。  
**需求**：规范 §8–9、§15。

**实施**：

1. 威胁测试：远程 HTTPS 页面调用宿主 `apps.launch` / `files.list` / `executeShell` 必须拒绝；`file://` 路径、恶意下载路径、ZIP 穿越、输入路径中的 Shell 特殊字符不越权。
2. 原文件保护测试：转换故障、进程异常、索引取消、ZIP 恶意输入，均不得破坏源文件。
3. Profile 隔离回归：Cookie、历史和工作空间互不串用；清理某 Profile 不清理其他 Profile。
4. 稳定性：实际 WebView2 五标签、100 次操作，综合操作连续 60 分钟；损坏 PDF/ZIP 不导致主进程崩溃。
5. 性能：Windows 11 固定机器记录 20 次冷/热启动、Shell-only/1-tab/5-tab/索引/转换资源；如目标不能满足，先分析再通过 ADR 提请确认，不能修改目标来“通过”。
6. 第三方许可与依赖清单、秘密扫描和日志脱敏审查。

**产出**：`docs/security/v0.1-threat-review.md`、`docs/performance/v0.1-baseline.md`、`docs/test-plan/v0.1-matrix.md`、可复现的测试脚本及问题清单。  
**GATE B**：没有 P0 安全 Blocker/Critical，真实 Windows 性能及稳定性报告齐全；否则阻断发布。

---

## 17. WF-15 — Windows 安装版、Portable ZIP、CI 和版本检查（GATE C）

**目标**：交付能在测试机实际安装和便携运行的 V0.1 包。  
**需求**：APP-008、ADR-004/013、规范 §16。

**实施**：

1. 使用 GitHub Actions Windows x64 runner 编译；生成 NSIS `.exe` 或 WiX `.msi` 安装包（首版至少一个安装格式可用；如条件允许两个），以及 `Toolbox-v0.1-windows-x64-portable.zip`。
2. Portable 为**目录化应用**：`app/` 与 `data/` 分离，`data/` 在升级时保留；不是超大型单 EXE。检查目标目录可写、系统 WebView2 Runtime 是否存在，给出安全明确的补救提示。
3. 不随 Core 启动/打包完整 FFmpeg、Adobe、AI 模型和所有未来模块；记录安装包、解压后目录大小与进程明细。
4. 可以检查 GitHub 最新版本并提示，但更新必须由用户确认；没有经过验证的签名/校验更新链路时只提供手动下载，不伪称自动更新已可用。
5. GitHub Release 资产产出 SHA-256 校验、许可证与依赖说明、已知问题、commit SHA；构建尽可能可追踪。
6. Windows 10 兼容要求实测仍处于受支持生命周期或 ESU 范围的目标环境；没有测试资源就写 `NOT VERIFIED`，不得写“完全兼容 Windows 10”。

**测试**：干净 Windows 11 VM/测试机安装、启动、退出、卸载；Portable 两个不同文件夹运行与迁移；WebView2 Runtime 检测；Windows Runner 编译；Windows 10 目标环境（若具备）。  
**GATE C**：安装包 + Portable 都能正确运行，用户数据安全，无未经授权的静默升级；未验证项明确披露。

---

## 18. WF-16 — V0.1 Preview 发布审查与 GitHub Releases（GATE D）

**目标**：完成证据驱动的 V0.1 Preview 发布，而不是把 CI 绿色当成产品全部完成。  
**需求**：规范 §15–16，AC-001…AC-015。

**实施**：

1. 为 `AC-001` 到 `AC-015` 建立逐项证据表，标 `PASS/FAIL/NOT VERIFIED`，对应 PR/commit、环境和测试结果。
2. 检查 V0.1 三核心模块全部 P0 已通过；Adobe/Apple Music/扩展等 SPIKE 不得计入 P0 完成率。
3. 生成 `CHANGELOG.md`、`docs/releases/v0.1-known-issues.md`、依赖和许可清单、用户安装和 Portable 数据说明。
4. 执行发布候选 `v0.1.0-preview.N` 的 CI 构建与实机回归。只有发布权限和前置条件满足时，才在 GitHub Releases 创建 Draft；**真正公开发布需项目所有者明确批准**。
5. 发布资产需要可验证校验信息和明确平台声明；若 Windows 10 未验证，release notes 明示。
6. 发布后标记待跟进的 V0.2 Backlog，不要自动开始后续大功能。

**GATE D 通过条件**：关键安全和数据完整性检查通过、所有适用的 P0 验收有证据、测试包可安装、用户明确批准公开发布。  
**如不通过**：只可输出阻断报告与修复任务，不得宣称 V0.1 已完成。

---

## 19. 特殊情况与失败分支

| 情形 | 必须执行的动作 | 禁止行为 |
|---|---|---|
| Codex Cloud 无 Windows GUI | 提交 Windows CI/人工测试步骤并记录 NOT RUN | 用 Linux 测试冒充 Windows 实测 |
| Tauri 多子 WebView 失败 | 根据 WF-02 测独立 Browser Host 并提交 ADR | 使用模拟页面或 iframe 伪装真正 WebView2 标签 |
| Profile 不能隔离 Cookie | 停止浏览器跨工作空间交付并调整宿主/Profile 模型 | 只在数据库里做 `profile_id` 标签 |
| 无法加载某第三方网站 | 显示失败并使用系统浏览器/官方客户端回退 | 绕过 OAuth/DRM/网站安全策略 |
| Apple Music 内嵌无法播放 | 输出 SPIKE 报告，提供官方客户端/浏览器回退 | 实现未经授权的 MusicKit 或破解播放 |
| PDF/ZIP 库许可或安全测试失败 | 改选许可可接受的库或保持任务 blocked | 合并高危依赖、取消安全测试 |
| 一项功能过大 | 拆小 PR：契约 → 实现 → Windows 验证 | 伪造完成或悄悄改范围 |
| Windows 10 环境缺失 | 标注兼容性 NOT VERIFIED，并阻断声称支持该系统 | 将 Windows 11 结果复制为 Windows 10 结果 |
| 性能预算未达到 | 输出记录与原因、优化或 ADR 申请 | 不测量、隐藏子进程或改口径 |
| 缺少外部账户/API Key | 使用显式未配置状态、受控测试夹具 | 写死个人凭据、公开私有 URL |
| Codex Skills 未发现 | 直接读取已提交仓库中的适用 `SKILL.md`（若可访问）并记录方式 | 假称已调用不可用 Skill |

---

## 20. 每个任务完成时的标准报告模板

将以下结构用于 Codex 的最终答复和 PR 描述。

```markdown
## Task
- ID: WF-XX
- Requirements: APP-xxx / WEB-xxx / FILE-xxx / ...
- Scope: 本任务做了什么；明确没有做什么

## Changed files
- `path/to/file`: 修改目的

## Evidence and tests
| Test / command | Environment | Result | Evidence |
|---|---|---|---|
| ... | Codex Cloud | PASS/FAIL/NOT RUN | commit / log |
| ... | Windows 11 | PASS/FAIL/NOT VERIFIED | CI / 本地日志 |

## Safety / compatibility / resources
- Permissions:
- Data migration:
- Performance or package impact:
- Rollback:

## Blockers and next action
- State: verified / awaiting-windows-validation / blocked
- Windows actions required:
- Next eligible task (only after Gate): WF-XX
```

### 20.1 进度台账示例（首次建立后由任务实际结果填充）

| Task | Initial status | Required gate |
|---|---|---|
| WF-00 | not-started | 规范已提交 |
| WF-01 | not-started | WF-00 |
| WF-02 | not-started | WF-01 Windows 可启动 |
| WF-03 | not-started | GATE A |
| WF-03.5 | not-started | WF-02 verified / GATE A pass + WF-03 verified |
| WF-04 | not-started | WF-03 + WF-03.5 verified（设计 PR 审查/合并） |
| WF-05 | not-started | WF-04 |
| WF-06 | not-started | GATE A + WF-03 + WF-03.5 verified（设计 PR 审查/合并） |
| WF-07 | not-started | WF-06 |
| WF-08 | not-started | WF-07 |
| WF-09 | not-started | WF-03 + WF-03.5 verified（设计 PR 审查/合并） |
| WF-10 | not-started | WF-09 |
| WF-11 | not-started | WF-09 + 依赖库验证 |
| WF-12 | not-started | WF-05 + WF-08 + WF-09 |
| WF-13 | not-started | WF-03 + 必要业务表已实现 |
| WF-14 | not-started | 所有 P0 业务任务完成 |
| WF-15 | not-started | GATE B |
| WF-16 | not-started | GATE C + 产品所有者审批 |

---

## 21. Windows 11 / VS Code 测试者操作清单

Codex Cloud 将需要实际 Windows 的任务在 PR 中标出；你可在自己的 Windows 11 电脑上按以下步骤操作，**不要把私人本地日志直接公开**。

1. 拉取目标 PR 分支，用 README 中锁定的 Node/Rust/MSVC 工具链安装开发依赖；确认 WebView2 Runtime 状态。
2. 运行仓库提供的 Windows 开发或测试命令，打开真正的 Toolbox 窗口，记录 commit SHA 与应用版本。
3. 对 WF-02：测试真实多标签、Profile Cookie 隔离、Portable 数据目录、20 次启动性能；将**脱敏后的**日志或测试结果贴回 PR。
4. 对 WF-04/05：至少发现和启动一个已经安装的真实软件，并用指定应用打开文件；不要测试任何不受信任的程序。
5. 对 WF-06/08：使用无需账户的 HTTPS 与 localhost 页面测试导航和下载；Apple Music 等外部服务单独标记为 SPIKE。
6. 对 WF-09/11：只在事先复制好的测试文件夹运行删除、移动、转换、ZIP 解压；验证原文件保留，禁用对私人原始素材的破坏性测试。
7. 对 WF-15：在干净目录测试安装版与 Portable，不把个人数据目录一起上传 CI artifact。
8. 将结果整理成 `PASS/FAIL/NOT VERIFIED`，记录发生问题的操作序列与日志；由 Codex 修复同一个任务后复测。

---

## 22. 项目所有者的阶段确认清单

- [ ] 首先提交 `TOOLBOX_SPEC.md` + `CODEX_WORKFLOW.md` 到 GitHub。
- [ ] `WF-00`：阅读生成的 `AGENTS.md`，确认没有越权或擅自加需求。
- [ ] `WF-01`：在自己的 Windows 11 实际启动程序。
- [ ] `WF-02`：审核多 WebView/备用宿主的 Gate A ADR，确认最终浏览器架构。
- [ ] `WF-03`：审核数据库、接口与权限边界。
- [ ] `WF-03.5`：确认 WF-02、WF-03 已验收；审阅信息架构、设计系统、页面/交互、Light/Dark/未来主题、组件/图标、动态模块导航与性能约束，验收设计交付后再进入 WF-04。
- [ ] `WF-05`：软件管理基础操作实际可用。
- [ ] `WF-08`：浏览器多标签、Profile、下载与 Web App 可用。
- [ ] `WF-11`：文件管理、全文索引、图片/PDF/ZIP 转换可用。
- [ ] `WF-14`：安全、性能、稳定性报告可信。
- [ ] `WF-15`：安装版、Portable 真实可用且数据不混淆。
- [ ] `WF-16`：确认发布内容，批准（或拒绝）公开发布 GitHub Release。

---

## 23. 官方技术参考（实现前核对所用版本）

- [Tauri 2 Webview API：子 WebView 需要 `unstable` 特性](https://v2.tauri.app/reference/javascript/api/namespacewebview/)
- [Tauri 2 Windows 安装与 WebView2 发行选项](https://v2.tauri.app/distribute/windows-installer/)
- [Microsoft WebView2 多 Profile](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/multi-profile-support)
- [OpenAI Codex：仓库级 AGENTS.md 的指引](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide)
- [OpenAI：AGENTS.md/Skills 应保持精简且任务相关](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)

**最终执行要求：每个 PR 只完成一个可审查任务；没有 Windows 真实测试证据不宣称 Windows 功能验证通过；没有通过 Gate 不进入后续阶段；一切以 `TOOLBOX_SPEC.md` 的 V0.1 P0、安全和性能边界为准。**
