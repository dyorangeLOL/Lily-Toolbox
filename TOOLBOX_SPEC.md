# Toolbox 产品与技术开发规范（TOOLBOX_SPEC）

> 状态：需求基线（Baseline）｜版本：1.0｜制定日期：2026-10-14  
> 产品：Toolbox — Windows 优先的模块化个人生产力工作台  
> 面向：产品所有者、Codex Cloud、Windows 本地开发者、CI 流水线  
> 约束：**本文不是 UI 设计稿，也不是已完成产品的能力声明。** `P0` 是 V0.1 必做，`P1/P2` 是后续版本需求，`SPIKE` 为先验证再决定的技术假设。

---

## 0. 执行摘要和冻结决策

Toolbox 定位为「**个人先用、稳定后公开发布**」的全能混合工作台：整合 Windows 桌面软件、Web Apps、文件管理、资料检索与格式转换，并为后续 AI、自动化、Adobe、手机端、插件与 Skills 预留规范化扩展能力。

### 0.1 三轮访谈确认的不可随意变更的决策

| 决策编号 | 决策 | 对实现的影响 |
|---|---|---|
| ADR-001 | Windows 优先；V0.1 覆盖 Windows 11 x64 和**仍有安全支持的** Windows 10 x64 配置 | Windows 10 普通版已结束常规支持；优先测试 ESU/LTSC 等合规配置 |
| ADR-002 | 第一版为可运行的技术预览版（Preview），但三个核心模块的基础操作必须真实可用 | 禁止用假数据、空点击或截图冒充验收完成 |
| ADR-003 | 优先 Tauri 2 + React/TypeScript + Rust；浏览器关键能力先做 SPIKE | 不为固守框架牺牲多标签功能与稳定性 |
| ADR-004 | 基于系统 WebView2 Runtime；安装包 + Portable ZIP | **Portable 是目录化发行**，不是所有东西合成一个可执行文件 |
| ADR-005 | V0.1 三大核心：桌面应用管理、浏览器/Web Apps、文件管理/搜索/基础转换 | 其他功能在独立 Backlog 中按版本推进 |
| ADR-006 | 浏览器按工作空间隔离 Profile，允许 Web App 覆盖 | 尽量使用共享 User Data Folder + 多 Profile，减少重复浏览器进程 |
| ADR-007 | 现有记账和摄影工作台使用已有 Web 应用，第一版仅作为可登录 Web App 集成 | 不重复开发账本和摄影业务；跨应用通信以后开发 |
| ADR-008 | 默认保留原文件；删除优先回收站；敏感操作确认 | 转换另存和批量任务预览是基础安全要求 |
| ADR-009 | SQLite 保存本地业务数据；浏览器数据独立保存；用户文件留在原位置 | 索引不是文件本体，不擅自迁移源文件 |
| ADR-010 | 内置模块先行，预留插件清单、接口和安全模型，V0.1 不接纳任意第三方插件 | 避免第一版运行未经信任的代码 |
| ADR-011 | 核心命令 API + 类型化事件；V0.1 不建云服务器 | 易于测试，降低维护及内存开销 |
| ADR-012 | Codex Cloud 负责通用开发测试；Windows CI/本地负责真实桌面验证 | Linux 云端模拟通过不等于 Windows 产品完成 |
| ADR-013 | GitHub Releases 发布测试版本；安装版可验证更新，Portable 手动替换 | 必须签名/校验更新资产；更新检查不得静默安装 |
| ADR-014 | **快速启动、稳定、低内存、按需加载是硬架构要求** | 核心 Shell 轻量，重型功能、转换引擎、桥接器、插件分包提供 |
| ADR-015 | 第一阶段 Apple Music 仅做 WebView2 网页嵌入播放的技术验证，失败可外部打开 | 无 Apple Developer Program，不开发自制 MusicKit 播放器 |
| ADR-016 | Adobe 深度解析独立验证，未证实兼容性前不得宣传“完整编辑” | V0.1 只做文件关联、启动、索引，不编辑原生工程内部结构 |

### 0.2 功能标签解释

- `P0/V0.1`：必须实现并通过验收。
- `P1/V0.2`：第二阶段目标；不阻塞 V0.1。
- `P2/V0.3+`：长期规划；允许重新评估实现。
- `SPIKE`：产出可复现实验、结果与继续/停止决策，不等于承诺正式产品能力。
- `NFR`：非功能性要求，每个版本均应遵守。

## 1. 产品目标、原则与非目标

### 1.1 产品目标

1. 一个入口管理桌面应用、网页应用、文件和项目；兼容 Windows 文件关联及应用状态检查。
2. 提供可实际使用的 WebView2 浏览器：多标签、收藏夹、历史、下载和分离的 Web App 工作空间。
3. 提供不破坏原文件的文件浏览、搜索、PDF/Markdown 全文索引、图片/PDF/压缩包基础工具。
4. 在不修改已有业务逻辑的前提下集成现有记账、摄影 Web Apps；让它们成为后续互联生态的一部分。
5. 功能扩展不会导致核心程序越装越臃肿；无需启用全部功能也能快速启动。
6. 准备未来移动端、AI、多步骤自动化、Adobe 插件、Skills 与数据同步的接口，但不在 V0.1 一次性开发完。

### 1.2 核心原则

- **Local-first**：离线可访问本地应用、浏览文件、搜索已有索引；不依赖厂商云服务才能启动。
- **Secure-by-default**：普通网页没有本地文件、Shell、任意进程启动权限；授予权限可撤销。
- **Modular-not-microservices**：进程上尽量简单，代码上严格模块化；只在性能/隔离需要时启动工作进程。
- **Lazy-by-default**：插件、索引器、转换器、媒体探测、Adobe、AI 等不在主进程启动链上初始化。
- **Data ownership**：源文件原地保留，用户可导出、备份、恢复并清理自己的数据。
- **No fake integration**：外部平台只在有受支持 API、明确授权或可验证的网页嵌入能力时宣称“集成”。
- **Evidence-driven**：性能、浏览器、登录、文件兼容必须由实测与 CI 证据证明。

### 1.3 明确不属于 V0.1 的功能

- 完整 Edge 扩展商店、广告拦截、翻译、阅读模式、网页自动化、任意用户脚本。
- Adobe PSD/PSB 图层编辑、PRPROJ 时间线修改、AEP/AEPX 合成编辑；独立的修图/剪辑器。
- 云同步、账号系统、远程开机/远程控制、自建后台同步服务器、手机端 App。
- AI API 聚合、本地模型运行、完整可视化工作流、开放的插件/Skills 执行环境。
- 所有格式的全兼容转换、Office 文档的高保真转换、音视频编码引擎内置于主程序。

## 2. V0.1 功能规格与验收

### 2.1 桌面核心与应用管理（APP）

| ID | 优先级 | 需求 | 可验证结果 |
|---|---|---|---|
| APP-001 | P0 | Windows 应用安装清单扫描（开始菜单快捷方式、可用的已安装程序登记信息等受支持来源） | 扫描得到实际程序；单个异常登记项不令程序崩溃 |
| APP-002 | P0 | 手动添加 EXE 或快捷方式、名称、图标、启动参数 | 保存后重启仍可打开；不支持的路径有错误提示 |
| APP-003 | P0 | 搜索、收藏、分组与最近启动记录 | 用户新增/修改后能被查询和持久化 |
| APP-004 | P0 | 按文件关联打开，或显式选择应用打开文件 | 支持系统关联及手动指定可执行文件；参数正确转义 |
| APP-005 | P0 | 应用运行状态（最佳努力）和一键启动 | 可识别已运行进程；无权读取时显示“未知”，不伪报未运行 |
| APP-006 | P0 | 常用快捷命令（固定、白名单动作） | 可执行预定义命令；不将用户字符串直接拼接为 Shell 命令 |
| APP-007 | P0 | 单实例启动、异常关闭恢复、基础设置 | 同一数据目录的重复启动不会破坏 SQLite 或浏览器 Profile |
| APP-008 | P0 | 安装版和 Portable 发行 | 安装版默认按用户安装；Portable 能在可写目录独立保存数据 |
| APP-009 | P1 | 系统托盘、开机启动、全局快捷键 | V0.1 可预留适配接口；V0.2 正式实现并处理权限与用户配置 |
| APP-010 | P1 | 更新检测及用户确认安装 | V0.1 可先提示 GitHub 版本；安装更新机制在签名测试通过后启用 |

**限制**：程序扫描不需要管理员权限；不要尝试遍历所有进程模块或解析所有已安装软件的私有数据库。不得提供关闭系统安全软件等功能。

### 2.2 浏览器与 Web Apps（WEB）

| ID | 优先级 | 需求 | 可验证结果 |
|---|---|---|---|
| WEB-001 | P0 | 使用 WebView2 加载 HTTPS 网页、localhost、本地可信打包 Web App | 三类来源均能正确识别并受各自权限策略约束 |
| WEB-002 | P0 | 多标签打开、切换、关闭及会话恢复 | 至少 5 个常见页面反复切换，关闭单一标签不影响其他标签 |
| WEB-003 | P0 | 收藏夹创建、修改、删除、按 Profile 管理 | 数据可持久化，切换 Profile 不串数据 |
| WEB-004 | P0 | 浏览历史记录、搜索、清理 | 不同 Profile 互相隔离；清理后的记录不再出现 |
| WEB-005 | P0 | 下载列表及开始/进度/完成/失败状态，保存路径选择 | 下载失败可见，下载路径必须通过安全验证 |
| WEB-006 | P0 | Web App 注册：URL/显示名/工作空间/首选打开方式 | 现有记账、摄影工作台只需填 URL 就可以注册和启动 |
| WEB-007 | P0 | 默认按工作空间隔离 Profile，单个 Web App 可覆盖 | 两个工作空间登录数据不共享；同空间共享符合配置 |
| WEB-008 | P0 | 网站无法嵌入或登录受限制时，手动回退外部浏览器/原生客户端 | 不绕过网站安全策略；用户能选“在系统浏览器打开” |
| WEB-009 | P0 | 防止普通远程网页直接调用原生 Tauri 命令 | 对未知域发起原生调用请求必须拒绝并记录可审计结果 |
| WEB-010 | SPIKE | 同一窗口下 Tauri 子 WebView 多标签方案 | 以 Windows 实测为准；如不可用转向独立浏览器宿主而非降为假标签页 |
| WEB-011 | SPIKE | Apple Music 网页登录/播放/后台音频兼容 | 记录实际测试；失败采用系统浏览器/官方客户端降级 |
| WEB-012 | P1 | 扩展、广告/跟踪拦截、DevTools、用户脚本、网页自动化、翻译和阅读模式 | 逐项验证 Edge 与 WebView2 差异，不宣称全部兼容 |
| WEB-013 | P2 | 临时会话、独立隐私容器与细粒度数据清理 | 统一建立 Profile 生命周期机制 |

**WebView2 重要边界**：它是 Edge Chromium 渲染运行时，**不是完整的 Microsoft Edge 浏览器**。浏览器侧边栏、商店扩展页面、收藏夹界面等需要 Toolbox 自行实现，第三方扩展不一定具有完整 UI 接口。

**多标签 Gate**：必须完成 `SPIKE-WEB-01`。允许选择以下技术路线之一：
1. 首选 Tauri 单窗口多子 WebView（需评估 `unstable` API 的稳定性、线程/事件循环及销毁流程）。
2. 如失败，保持 Tauri Shell，增加一个**按需启动**的独立 Windows WebView2 Browser Host（Win32/.NET/WinUI 的具体框架由实验决定），提供窗口附着/标签生命周期/命令协议。
3. 若仍不可行，提交 ADR 评估更换宿主框架；不得隐藏风险并声称多标签已通过。

### 2.3 文件管理、搜索与转换（FILE）

| ID | 优先级 | 需求 | 可验证结果 |
|---|---|---|---|
| FILE-001 | P0 | 浏览文件夹、排序、搜索当前目录、查看基本信息 | 大文件夹可增量加载；权限不足不导致程序退出 |
| FILE-002 | P0 | 复制、移动、重命名、新建文件夹、回收站删除 | 操作有进度、取消及冲突处理；删除默认进入回收站 |
| FILE-003 | P0 | 常用文件夹自动索引（桌面、文档、下载、图片、视频）和自定义路径 | 首次运行提示范围并可暂停/排除；系统保护目录不扫描 |
| FILE-004 | P0 | 文件名、路径、类型、大小、mtime 等结构化搜索 | 首次索引完成后可搜索；增量变更反映到结果 |
| FILE-005 | P0 | PDF 可提取文本 + Markdown 内容的全文搜索（FTS5） | 以可提取文字的 PDF 与 Markdown 测试文件验证查询命中 |
| FILE-006 | P0 | 安全的扫描队列、取消/暂停、索引错误记录 | 大量文件不阻塞主线程，关闭后可续作，不重复大规模全盘扫描 |
| FILE-007 | P0 | 图片格式基础转换：PNG/JPEG/WebP（其他格式能力检测） | 默认另存、输出可打开、转换失败不覆盖原件 |
| FILE-008 | P0 | PDF 合并、拆分、图像转 PDF（依赖经验证的文档库） | PDF 文档正确页数、页序；加密/损坏 PDF 有明确提示 |
| FILE-009 | P0 | ZIP 解压与创建；识别已支持归档 | 防 Zip Slip 路径穿越；拒绝超限膨胀、异常软链接 |
| FILE-010 | P1 | HEIC、音视频格式转换、RAR/7z 等扩展格式 | 作为独立可选模块；第三方工具许可需核实 |
| FILE-011 | P1 | DOCX/Office 全文索引和文件转换 | Office 高保真转换为需验证任务，不依赖必须安装 Office 的假设 |
| FILE-012 | P1 | 图片 EXIF、视频编码和音频元数据、标签与项目索引 | 保留扩展字段，不与最初的基础索引相互耦合 |
| FILE-013 | P1 | 批量重命名、批量分类、查重、标签 | 提供预览/撤销方案，避免误操作 |
| FILE-014 | P2 | 统一摄影素材库、来源引用/复制策略、素材工作流 | 可按项目决定引用或导入，绝不强制移动用户素材 |

**明确降级**：扫描件 PDF 不提供 V0.1 OCR；复杂 PDF 字体、损坏 PDF、密码文件可提示不支持。图片 HEIC、Office 转换、音视频 FFmpeg 转码虽在总需求中，**不属于 V0.1 必需的基础转换**，并保留可选模块入口。压缩包 V0.1 首选 ZIP；其他格式验证后扩展。

### 2.4 外部软件联动与既有 Web Apps（INT）

| ID | 优先级 | 需求 | 可验证结果 |
|---|---|---|---|
| INT-001 | P0 | Windows 应用启动、文件关联、路径和受控启动参数 | 不使用 shell 字符串拼接，参数以数组传递并检查路径 |
| INT-002 | P0 | Adobe Photoshop、Premiere Pro、After Effects 程序识别与启动 | 未安装时明确告知；已安装时可打开软件 |
| INT-003 | P0 | 识别 `.psd/.psb`、`.prproj`、`.aep/.aepx` 文件关联 | 文件基本分类与系统打开工作正常；不声称已解析内部项目 |
| INT-004 | P0 | 应用列表内注册“记账”和“摄影工作台”两个现有 Web Apps | URL 配置可持久化；未提供实际网址时不写死假地址 |
| INT-005 | P0 | 桌面文件到指定应用/网页的显式操作：打开文件或复制安全路径 | 普通网页无法静默获得真实路径 |
| INT-006 | P1 | 记账和摄影工作台与 Toolbox 的结构化数据通信协议 | 通信前明确双方版本、授权、允许字段和 origin |
| INT-007 | SPIKE | Adobe 工程读取：PSD 图层 / PR 序列 / AE 合成 | 可行性研究与样例工程验证，之后决定独立解析还是官方插件 |
| INT-008 | P2 | 通过 Adobe 官方插件 API 修改原生项目 | 根据 Photoshop/Premiere/After Effects 各版本受支持接口实现 |

**文件扩展名纠正**：Premiere Pro 通用原生工程为 `.prproj`，不是 `.prpro` 或 `.prpro​j` 的其他拼写。Adobe 接口随着宿主版本更新，开发前需核对官方当前文档。

### 2.5 仅预留，不开发完整功能的模块

| 模块 | 优先程度 | 计划内容 |
|---|---|---|
| 小工具 | P1 | 剪贴板历史、截图/录屏、计算器、单位换算、时间工具、笔记、待办、日历 |
| 个人记账 | 已有 Web App | 首期内嵌；不重写其业务数据库；后期经授权联动 |
| 摄影工作台 | 已有 Web App | 首期内嵌；后期素材库与项目任务联动 |
| 开发者中心 | P1 | Git/GitHub、终端/PowerShell、JSON/Markdown/正则、HTTP API 调试 |
| AI 中心 | P1/P2 | 第一阶段仅 Web Apps（ChatGPT/Claude/Gemini）；后期在线 API、自带密钥、本地模型 |
| 自动化 | P1/P2 | 单步固定命令 → 触发器/调度 → 多步骤工作流；自定义脚本单独授权 |
| 插件与 Skills | P2 | 插件发现/签名/隔离/更新/权限；区分“读取 Skill 指令”和“执行本地能力” |
| 手机端 | P2 | 记账/摄影 Web Apps、文件/笔记/待办、电脑任务状态、经授权远程触发 |
| 同步 | P2 | 本地优先；可选账号/云盘同步，另做端到端数据模型与冲突解决设计 |

## 3. V0.1 架构及进程模型

### 3.1 候选技术栈

| 层 | 方案 | 状态 |
|---|---|---|
| 桌面 Shell | Tauri 2（Rust） | 选定首选；浏览器独立验证 |
| 前端 | React + TypeScript + Vite | 选定 |
| 状态/校验 | Zustand、Zod | 候选；不让状态库耦合核心领域逻辑 |
| 组件/样式 | Tailwind CSS + Radix/shadcn；图标 Lucide | 仅技术储备，**本文件不设计 UI** |
| 浏览器 | Windows WebView2 Evergreen | 选定优先内核 |
| 数据库 | SQLite + WAL + FTS5 | 选定 |
| 文件系统 | Rust 的标准库/Windows 安全 API | 选定；回收站等用受支持平台接口 |
| 图片 | Rust 图片库或受支持的 native codec | 先验证格式与色彩一致性 |
| PDF | 经过许可和兼容性测试的 PDF 引擎 | 选型待定；不自写 PDF 解析器 |
| ZIP | 经验证的归档库 | P0 |
| 音视频 | 外置 FFmpeg 工具包（按需安装） | P1；需遵守其许可与发行要求 |
| 测试 | Rust test、Vitest、Playwright、Windows UI/集成测试 | 选定测试方向 |
| CI | GitHub Actions：通用检查 + Windows x64 打包 | 选定 |

### 3.2 不把所有功能塞进一个 EXE：模块化单体 + 可选 Sidecar

**启动链只包含**：Toolbox Shell、配置管理、命令路由、模块清单、轻量 SQLite 打开、必要 WebView2 初始化。其余模块**按需加载**。

建议的运行边界：

```text
Toolbox.exe                  # 最小桌面宿主：Shell + Core + 受控命令
├── Windows WebView2 runtime # 系统提供；网页打开时才实际加载所需资源
├── toolbox-indexer.exe      # 文件索引后台 worker；索引任务启用时按需运行
├── toolbox-converter.exe    # 常用转换 worker；转换任务时运行
├── toolbox-browser-host.exe # 仅当 Tauri 多子 WebView 不满足要求时启用
└── optional-tools/         # 大型 PDF/媒体/Adobe/AI 插件与资源（未来）
```

**注意**：这是逻辑拆分方案，不要求 V0.1 一次就产生所有可执行文件。若一个轻量 Rust 线程池比独立进程更节省资源，可留在核心进程；单独 Sidecar 仅用于高依赖、可能崩溃、不可信输入或需独立生命周期的任务。核心 Shell 内**不应常驻多个空闲后台进程**。

分包规则：

1. `base`：Shell、应用管理、浏览器管理、基础文件操作、SQLite、轻量搜索和最少运行依赖。
2. `builtin-light`：较小的内置扩展；可与 base 同安装，但延迟初始化。
3. `optional-pack`：音视频引擎、附加编解码器、Adobe bridge、AI/模型引擎；需要时单独下载或离线导入。
4. 每个可下载工具包有 `id/version/platform/architecture/dependencies/checksum/signature/licenses` 元数据；下载后校验，安装失败不影响核心。
5. 不承诺所有模块能在应用运行时热卸载；优先保证模块生命周期安全与进程稳定。

### 3.3 依赖方向与解耦

```text
apps/desktop (Tauri host)
    ├── core/domain               # 不依赖 WebView、UI、具体插件实现
    ├── core/commands + events    # 受控 API、授权与路由
    ├── modules/app-manager
    ├── modules/browser
    ├── modules/file-manager
    ├── modules/search
    ├── modules/conversion
    └── integrations/windows / future

frontend React (通过受控 invoke/events)
    └── 不能直接读取 SQLite、启动任意程序或暴露密钥

remote Web Apps (WebView2)
    └── 默认没有 Tauri 原生桥；未来仅可信 origin + 用户许可 + 最小 API
```

依赖反转要求：领域接口位于 core；模块实现依赖接口；前端仅依赖共享类型；不要让浏览器服务直接写应用管理模块的内部表。

### 3.4 进程及内存生命周期规则

- 启动不枚举全部磁盘、不加载所有收藏夹内容、不预加载所有网页标签。
- 浏览器后台标签：支持卸载/休眠策略；已使用音频播放或执行用户任务的标签不能无提示销毁。
- 尽量使用**一个共享的 WebView2 User Data Folder（UDF）加多个 Profile**，不要为每个网页单独创建整个浏览器进程树。
- 索引器：低优先级任务、增量扫描、暂停/取消、可恢复任务状态；系统忙或省电模式自动减速（策略可配置）。
- 转换器：进程单次或任务队列使用；闲置自动退出；资源和执行并发有限制。
- 退出：先等待/取消任务，安全关闭 WebView、flush SQLite/WAL，再释放锁；强制退出后须能恢复。
- 禁止在 UI/浏览器主线程执行大文件递归扫描、解压或视频转换。

## 4. 浏览器实现细则及降级原则

### 4.1 Profile 设计

- `workspace` 默认引用一个 WebView2 Profile；Web App 可选择 `inherit`、指定已有 Profile 或独立 Profile。
- 浏览历史、收藏夹、标签会话在 Toolbox SQLite 中按照 `profile_id` 逻辑隔离；Cookie、缓存、网站权限存储在 WebView2 Profile。
- `profile_id` 为内部稳定 UUID，不直接使用用户输入为目录名。
- 清理 Profile 时先关闭依赖它的所有浏览控件；数据库记录和 WebView2 存储一致处理。
- Portable 搬迁至新电脑时，登录 Cookie、DPAPI 保护的凭据和 DRM 不保证可迁移；无感迁移不得作为验收要求。

### 4.2 导航与网页生命周期

- 导航事件：`created → loading → committed → completed/failed`；下载另有任务状态机。
- 不信任远程页面提供的“网页标题”和路径；展示时统一转义，限制 URL scheme。
- 只允许受支持的导航 scheme（如 https、http://localhost 的受控场景）；`file://`、自定义协议、外部应用跳转需显式安全策略。
- OAuth 和不允许嵌入的登录使用系统浏览器回退；不能伪装 User-Agent 或绕过站点安全策略。
- DevTools、扩展、用户脚本和广告拦截在 V0.1 只有 Feature Flag/接口占位，不等于可交付功能。

### 4.3 Apple Music 与第三方网页的能力表

- `URL 打开`：应支持；网页是否允许登录/播放由实测决定。
- `音频播放`：SPIKE，测试登录、DRM、后台播放、切换标签后状态。
- `MusicKit 自制播放器`：不在开发范围，因为没有当前所需的 Apple Developer Program 资格与开发者凭据。
- `访问第三方网页 API`：仅通过官方开放 API 和受支持授权方式；不抓取或绕过权限。

## 5. 文件、转换与搜索架构

### 5.1 文件安全边界

- 所有文件路径通过规范化验证；符号链接、junction、目录穿越与大小写特殊情况需要单独测试。
- 复制/移动/重命名支持重名冲突：跳过/重命名/覆盖前确认；批量操作先显示预估修改清单。
- 删除默认回收站，不可用时提示，不能静默永久删除。
- 转换默认输出至原目录的带后缀新文件或用户指定输出目录；源文件必须保留。
- 向第三方网页暴露文件只能经显式选择/授权，不直接传递真实本地路径。
- 安全防护重点：ZIP 路径穿越、解压炸弹、损坏媒体解析、超大文件、相对路径、软链接、TOCTOU、路径长度。

### 5.2 文件搜索工作流

```text
授权/选择扫描范围
  → 首次轻量枚举文件系统元数据
  → SQLite 文件记录更新
  → 后台提取 PDF/Markdown 可提取文本
  → FTS5 建立倒排索引
  → 文件修改/删除/重命名增量更新
  → 搜索结果展示与定位
```

- 自动扫描 Windows 桌面/文档/下载/图片/视频等常用文件夹，首次运行清楚告知并允许排除路径。
- 可以设置大小、扩展名、隐藏目录、临时目录、敏感目录忽略清单；避免对整个系统盘无条件扫描。
- PDF 的扫描版图片 OCR 属于后续功能；文档加密或提取失败只索引元数据并记录原因。
- 以文件系统元数据/稳定 File ID（若可靠可用）结合路径和更新时间做增量判定；避免每次启动全量哈希。
- 索引过程读取有限并发，不在 UI 主线程跑；结果能标识索引状态和更新时间。

### 5.3 转换器统一协议

转换任务至少包括：`id`、`format`、`inputs[]`、`output_directory`、`options`、`overwrite_policy`、`state`、`progress`、`error_code`、`created_at`。状态：`queued/running/succeeded/failed/cancelled`。

- 转换包的格式能力需要动态查询，避免界面展示实际不存在的编码器。
- 执行前校验输出磁盘空间、输出路径可写性、原文件保护策略。
- 如果取消任务，清理自己的临时文件，不删除用户原始文件。
- 非自有许可的外部引擎必须在发行清单里记录许可、来源、版本及必要声明。

## 6. 本地数据与数据库设计

### 6.1 数据位置

**安装版**：配置、数据库、日志位于当前 Windows 用户的应用数据目录；WebView2 UDF 使用独立子目录。程序二进制安装在系统建议位置，不和用户文件混放。

**Portable**：以 `ToolboxPortable/` 为根目录，二进制及可选扩展在 `app/`，个人数据优先写到 `data/`。只在该目录可写时启用 Portable 完整持久化；位于 Program Files/只读 U 盘时给出清晰报错或允许显式选择其他数据目录，不能静默改为系统目录。

```text
ToolboxPortable/
├── app/
│   ├── Toolbox.exe
│   ├── runtime/                 # 非强制；不默认打包 WebView2 Runtime
│   ├── modules/
│   └── tools/                   # 可选工具包
├── data/
│   ├── config/
│   ├── db/toolbox.sqlite
│   ├── webview/                 # WebView2 UDF；按 Profile 隔离
│   ├── cache/
│   ├── logs/
│   ├── backups/
│   └── module-data/
└── licenses/
```

### 6.2 SQLite 初版逻辑实体

| 表 / 逻辑实体 | 主要字段与约束 |
|---|---|
| `schema_migrations` | `version, applied_at, checksum`；迁移具备顺序和失败回滚策略 |
| `settings` | `key, value_json, scope, updated_at`；非敏感配置 |
| `installed_apps` | `id, name, launch_target, arguments_json, icon_ref, discovered, last_seen_at` |
| `app_favorites` | `app_id, sort_order, group_id` |
| `app_launch_history` | `id, app_id, launched_at, outcome` |
| `workspaces` | `id, name, default_profile_id, updated_at` |
| `browser_profiles` | `id, internal_name, webview_profile_ref, data_policy, created_at` |
| `web_apps` | `id, name, url, workspace_id, profile_override_id, open_mode, enabled` |
| `browser_bookmarks` | `id, profile_id, url, title, parent_id, created_at` |
| `browser_history` | `id, profile_id, url, title, visited_at`（个人数据） |
| `browser_tabs` | `id, workspace_id, profile_id, url, sort_order, recoverable_state` |
| `browser_downloads` | `id, profile_id, source_url, output_path, state, bytes_done, error` |
| `indexed_roots` | `id, path, enabled, include_rules, exclude_rules, last_scan_at` |
| `file_index` | `id, path, parent_path, extension, size, mtime, indexing_state, content_status` |
| `file_fts` | `FTS5` 虚拟表，使用稳定 `file_index` 关联项；避免重复存储原文件 |
| `conversion_jobs` | `id, module_id, state, request_json, result_json, created_at, finished_at` |
| `modules` | `id, version, kind, enabled, installed_path, manifest_version` |
| `permission_grants` | `id, subject_type, subject_id, capability, resource_scope, grant_type, revoked_at` |
| `audit_events` | `id, event_type, subject_id, outcome, timestamp, safe_details_json` |
| `backup_catalog` | `id, backup_kind, file_path, created_at, schema_version, verified_at` |

要求：开启 WAL、外键约束、事务、参数化查询；合理索引；向前升级提供 Schema Migration；不要把 OAuth Token、密码、完整 Cookie 放在一般业务表里。具体 SQL 类型与 FTS 同步方式由实现阶段确定并记录。

### 6.3 备份、敏感数据与恢复

- `Standard Backup`：业务数据库一致性快照、设置、工作空间、应用清单、收藏夹和经过脱敏的模块配置；不包含全部用户文件，也**不默认**打包浏览器 Cookie/登录状态。
- `Encrypted Backup`：用于经用户选择的敏感配置；采用经过审计的密码密钥派生 + 认证加密方案，必须有完整恢复测试；不得只压缩为带密码 ZIP 就声称安全。实现前需选定成熟密码库和 KDF/AEAD 参数。
- 本机秘钥可使用 Windows DPAPI 或系统凭据机制，但其跨电脑 Portable 性能和可迁移性必须单独说明；用户需要重新登录外部网站可能是正常结果。
- V0.1 至少交付**普通备份、恢复、备份完整性检查**；加密敏感备份作为 `P1`，不应在未验证安全性时出现在可用功能清单。
- 恢复操作先检验版本/完整性，再在测试数据库或临时目录演练，不覆盖现有数据直到用户确认。

## 7. 模块接口、事件和扩展契约

### 7.1 模块定义（可测试的契约，非代码实现）

每个模块清单至少声明：

- `id`：反向域名或稳定前缀，如 `toolbox.browser`。
- `version`：SemVer。
- `api_version`：Toolbox 模块 API 版本。
- `kind`：`builtin/optional/sidecar`（V0.1 不支持第三方任意插件）。
- `features[]`：提供的功能和能力标识。
- `permissions[]`：需要的最小能力范围。
- `dependencies[]`：工具包或其他模块的版本约束。
- `entry`：已验证、受信任的入口；可选模块需签名与校验。
- `lifecycle`：`registered → available → activated → deactivated/failed`。

### 7.2 统一命令规范

最初的核心命令（内部受控调用）包含：

| 命令族 | 样例 | 权限与边界 |
|---|---|---|
| `apps` | `apps.list`, `apps.register`, `apps.launch`, `apps.openFile`, `apps.status` | 仅可信前端；启动目标受控 |
| `browser` | `browser.open`, `browser.closeTab`, `browser.listProfiles`, `browser.bookmark` | Web Profile 与工作空间显式绑定 |
| `files` | `files.list`, `files.search`, `files.copy`, `files.move`, `files.trash` | 先验证路径及授权范围 |
| `convert` | `convert.capabilities`, `convert.start`, `convert.cancel`, `convert.status` | 重型操作队列异步执行 |
| `modules` | `modules.list`, `modules.capabilities` | 初期只读取已内置模块能力 |
| `backup` | `backup.create`, `backup.restore`, `backup.verify` | 恢复需用户确认，敏感路径不暴露给网页 |

命令请求应包括 `request_id`、`api_version`、`caller_context`、`params`；响应包括 `success`、`data` 或 `error { code, message, retryable }`。长任务用 `job_id` 和事件推送，禁止单个同步 IPC 调用占用主线程。

### 7.3 事件主题

推荐：`app.launched`、`browser.tabChanged`、`browser.downloadProgress`、`file.operationCompleted`、`index.scanProgress`、`convert.jobChanged`、`permission.changed`、`backup.completed`。

- 事件必须有 `event_id, timestamp, source_module, schema_version`；事件数据只包含需要共享的最少信息。
- V0.1 不需要持久化所有事件；需要可靠执行的任务状态写入 SQLite。
- 每个事件订阅者有生命周期解绑机制，防止页面关闭后监听器泄漏。
- 不要通过全局事件总线携带密钥、Cookie、用户文件全文或完整敏感路径。

### 7.4 Web App 桥接协议（P1，仅定义）

第一版 Web App **默认无原生桥接权限**。后续仅对明确登记为 `trusted-web-app` 的应用、逐项声明特权 API，经用户确认，提供最小授权通道。

后续消息需包含：`protocol_version`、`app_id`、`origin`、`nonce/request_id`、`action`、`payload`；宿主校验 origin、关联 WebView、权限范围、请求结构、结果大小和请求速率。禁止从任意公开网页中注入一个万能 `executeShell` 函数。

### 7.5 Skills 定义（P2）

- Skill 是可被读取的约定/说明，不自动等于可执行脚本或已安装插件。
- 将来的 Skills 运行必须经 Capability Broker 转换为受控系统动作，显式检查模块权限。
- Codex Cloud 中仓库内 `.agents/skills/*/SKILL.md` 是开发指导资源，**不与用户本地 Toolbox 插件运行时混为一谈**。
- 第三方插件 SDK、安装市场、远程更新与签名安全评估在 P2 才开放。

## 8. 安全与权限模型

### 8.1 信任等级

| 等级 | 来源 | 允许范围 |
|---|---|---|
| T0 | Rust 宿主与签名内置模块 | 仍受命令白名单、文件作用域和用户确认约束 |
| T1 | Toolbox 打包的本地可信前端 | 通过受控 Tauri API 获取必要权限 |
| T2 | 用户明确登记、经授权的 Web App（未来） | 按 `origin + app_id + capability + resource scope` 授权 |
| T3 | 任意远程网页、未审核插件、来源不明脚本 | 无本地命令/文件/凭据权限；仅网站自己的浏览器权限 |

### 8.2 权限机制

- V0.1 只提供固定、有限的系统动作；网页不能调用通用终端、任意 PowerShell、进程注入或操作其他应用私有数据。
- 后续权限可采用“首次请求确认、同范围持续有效”，但授权必须支持撤销，并可限制单次、目录、域名、模块。
- 更改授权范围（例如单文件提升到整盘）必须重新提示。
- 所有高风险操作记录脱敏的审计事件：时间、主体、动作、授权依据、结果。
- 文件对话框取得的授权不能自动扩张为整个磁盘；Symlink 和 Junction 需要额外防护。
- 可选包下载要求 TLS、校验哈希及发行者身份；不执行下载目录下未经验证的二进制。
- 在便携版中，本地数据加密与凭据迁移遵循 Windows 用户身份；不承诺 USB 随意插到别的电脑还能自动登录。
- 日志不得记录密码、Token、完整 URL 查询参数或文档全文；提供诊断日志脱敏选项。

### 8.3 安全验收必须覆盖

1. 任意外部 HTTPS 页面尝试调用 `apps.launch` / `files.list` 返回拒绝。
2. 可信应用撤销授权后立即无法继续调用对应能力。
3. 下载路径穿越、ZIP 恶意路径、极大压缩比、符号链接循环可拒绝或限制。
4. 传递包含空格、引号、特殊符号的路径时不会触发命令注入。
5. 故障日志、普通备份不含明文密钥与用户登录 Cookie。
6. 恶意/损坏转换输入最多使对应工作任务失败，不应带崩桌面 Shell。

## 9. 性能、稳定性和启动预算（NFR）

### 9.1 重点硬约束

- **无主窗口首屏前的全盘扫描、全应用探测、远程网络同步或可选包下载。**
- V0.1 核心启动路径不得依赖 Adobe、FFmpeg、AI 模型及额外云服务。
- 默认后台仅保持用户明确启用的功能；不启动一堆空闲 Sidecar。
- 关闭全部文件任务后索引/转换任务不应持续占用 CPU；任务队列能取消与恢复。
- 发生可恢复的 WebView/插件/转换进程异常时，要显示故障信息并支持重新初始化，不使整个工作台损坏。

### 9.2 性能基准（工程目标，不是对任意硬件的承诺）

测试基线：Windows 11 x64、SSD、至少 16 GB RAM、常见四核以上 CPU、系统 WebView2 已安装；发布前记录准确硬件、系统版本、测量脚本和冷/热启动定义。

| 指标 | V0.1 目标 | 测量方式 |
|---|---|---|
| 冷启动到主窗口首次可交互 | P90 ≤ 3 秒（目标） | 连续至少 20 次独立运行；独立计入 WebView2 必要初始化 |
| 热启动到首次可交互 | P90 ≤ 2 秒（目标） | 相同机器 20 次运行 |
| 无外部网页标签的 Shell 空闲私有内存 | **提出 ≤ 300 MB 的初始目标**，需在 M0 SPIKE 上冻结或调整 | 统计 Toolbox + 必需 WebView2 子进程的 Private Bytes，并给出子进程明细 |
| 稳定空闲 CPU | 60 秒均值 < 2%（目标） | 无活动下载/扫描/转换时统计进程树 |
| 可选工作进程空闲状态 | 0 个无任务却常驻的转换进程 | 任务结束后观察进程树 |
| 启动期间磁盘 IO | 不读取全部索引文件正文 | 事件跟踪/日志确认 |
| 常规稳定性 | 60 分钟典型操作无非预期主程序崩溃 | 覆盖标签切换、文件操作、搜索与转换 |

**指标说明**：网页本身（例如 AI 网站、Apple Music）消耗的内存随站点内容变化，不能用固定总 RAM 数字保证所有场景。应分别披露 `Shell only`、`1 tab`、`5 tabs`、`indexing active`、`conversion active` 指标和性能回归。若 M0 显示初始目标不切实际，需在 ADR 中记录实测依据并经产品所有者确认新阈值；不得通过隐藏进程或人为延迟计时达标。

### 9.3 可维护性与工程质量

- Rust 错误必须向上转换为稳定错误码；禁止 `unwrap()` 处理外部输入路径的常规失败。
- TS 启用严格类型；RPC 请求/响应严格 Schema 校验；避免大型全局状态对象。
- 新模块默认不增加主程序的启动初始化任务；启动性能回归超过冻结基线的 15% 需要分析。
- 存在破坏性数据库迁移时建立升级备份和回滚说明；对旧 Portable 数据的升级要有迁移测试。

## 10. 仓库结构与职责分界

以下为**推荐结构**；具体 Cargo Workspace 与构建脚本可在第一个 PR 调整，但需要保持职责清晰。不要在同一目录混杂所有业务。

```text
Toolbox/
├── README.md
├── TOOLBOX_SPEC.md               # 本文：需求唯一基线
├── AGENTS.md                     # 后续由本规范生成的 Codex 执行约束
├── CHANGELOG.md
├── LICENSE                       # 公开发布前决定许可证
├── docs/
│   ├── adr/                      # Architecture Decision Records
│   ├── architecture/
│   ├── api-contracts/
│   ├── security/
│   ├── test-plan/
│   ├── performance/
│   └── spikes/                   # 实验与基准结果
├── .agents/
│   └── skills/                   # 仓库内部的开发型 Skills（可选）
├── .github/
│   ├── workflows/
│   │   ├── ci.yml
│   │   ├── windows-build.yml
│   │   └── release.yml
│   └── ISSUE_TEMPLATE/
├── apps/
│   ├── desktop/
│   │   ├── src/                  # React 前端
│   │   ├── public/
│   │   ├── src-tauri/            # Tauri host、权限、打包配置
│   │   └── tests/
│   └── mobile/                   # 预留，V0.1 不创建运行时实现
├── crates/
│   ├── toolbox-core/            # 领域模型、错误、统一能力接口
│   ├── toolbox-storage/         # SQLite、迁移、Repository
│   ├── toolbox-apps/            # 本地软件发现与启动
│   ├── toolbox-browser/         # 标签/Profile/下载服务接口
│   ├── toolbox-files/           # 安全文件操作
│   ├── toolbox-index/           # 索引、搜索、FTS5
│   ├── toolbox-convert/         # 转换器协议和基础转换
│   ├── toolbox-permissions/     # Capability/Grant/审计
│   └── toolbox-module-registry/ # 模块清单，首版仅内置
├── packages/
│   ├── shared-types/            # TS DTO、JSON Schema
│   └── module-sdk/              # 仅接口草案；V0.1 禁止第三方执行
├── sidecars/
│   ├── indexer/                 # 若 SPIKE 决定进程隔离
│   ├── converter/               # 若 SPIKE 决定进程隔离
│   └── browser-host/            # 多 WebView 备用独立宿主
├── modules/
│   ├── builtin/
│   ├── optional/
│   └── manifests/
├── tools/
│   ├── packaging/
│   ├── benchmarks/
│   └── dev-scripts/
├── fixtures/
│   ├── files/
│   └── browser/                 # 只放不含隐私/凭据的测试样例
└── tests/
    ├── integration/
    ├── windows-e2e/
    ├── migration/
    ├── security/
    └── performance/
```

**不要机械地一次性创建全部空目录或 10 个空 crate**。M0~M2 按实际实现逐步落地，保持接口和分层约束即可。

## 11. 版本路线与范围冻结

### 11.1 V0.1（Preview）：仅三个核心模块

**必须**：
- Windows Shell、基础设置、安装版与 Portable 发行、单实例。
- 软件扫描/手动添加/搜索/收藏/启动/指定应用打开文件/基础状态。
- Edge WebView2 多标签（通过 SPIKE 确定稳定实现）、收藏夹/历史/下载、URL Web Apps、工作空间 Profile 隔离、外部浏览器回退。
- 文件浏览/复制/移动/重命名/回收站删除、常用目录索引、文件名/元数据搜索、PDF 与 Markdown 全文搜索。
- PNG/JPEG/WebP 基础图片转换、PDF 合并/拆分/图像转 PDF、ZIP 解压/创建。
- SQLite 迁移、正常备份/恢复、日志、基本权限防护、性能指标记录、Windows CI 打包及 GitHub Releases 测试发布。

**仅接口/实验**：模块清单、未来 Web App 数据桥、浏览器扩展位、Apple Music 浏览与播放兼容、Adobe 解析、可选 Tool Pack 分发协议。

### 11.2 V0.2（Integrated）：效率工具 + 高级文件能力

- 广告/跟踪拦截与扩展兼容性逐项测试，DevTools/脚本按权限开放。
- HEIC、媒体编解码、更多压缩包格式、DOCX/Office、EXIF/媒体元数据、标签/查重、批处理与分类。
- 剪贴板、截图/录屏、笔记/待办/日历、计算/单位/时间、开发者工具基本集成。
- 单步命令和固定触发器，托盘/开机启动/全局快捷键、安装版经过签名校验的更新。
- 根据需求增加敏感数据加密备份；建立 Web Apps 受控桥通信。

### 11.3 V0.3（Connected）：跨软件自动化与 AI

- AI API（明确用户提供密钥与成本）和本地模型适配；自动化调度、文件触发、条件与多步骤工作流。
- Adobe 解析或官方插件的验证成果进入正式产品；摄影素材项目联动；记账应用受控数据交换。
- 可选模块安装/卸载/校验；个人脚本按权限运行；模块日志和执行历史。

### 11.4 V1.0（Stable）：适合公开发布

- 稳定性、安全审计、迁移/备份/恢复验证、性能达标、可升级与可回滚、安装和卸载完整体验。
- 充分稳定的插件/Skills SDK、文档、兼容性矩阵。
- 可选云端同步与移动端方案通过独立 ADR 后再进入 V1.x；不强行绑定 V1.0 发布。

### 11.5 变更管理

- 任何新增的 P0 必须先说明用户价值、依赖、测试、维护成本及对启动/包体积的影响。
- 如果 P0 模块出现技术阻碍，必须先提交验证报告和替代方案，不能悄悄删除需求。
- 每个版本提交一份逐项需求状态（`not-started/in-progress/verified/blocked/deferred`）。

## 12. 技术验证任务（SPIKE）与进入开发的 Gate

| SPIKE | 验证问题 | 最小实验 | Gate 决策 |
|---|---|---|---|
| SPIKE-WEB-01 | Tauri 多子 WebView 标签页是否稳定？ | Windows 11 上创建/切换/关闭 10 个真实页面，连续 100 次操作，并测试窗口缩放、Profile、下载 | Pass：采用 Tauri 子 WebView；Fail：验证独立 Browser Host |
| SPIKE-WEB-02 | 多 Profile 是否隔离且内存可控？ | 两个工作空间访问同站点，验证不同 Cookie；测共享 UDF + 两 Profile 进程树 | 确定 Profile 实现与资源预算 |
| SPIKE-WEB-03 | 系统浏览器回退与登录兼容性？ | 测外部登录、回退打开、localhost 和自有 Web App | 不支持内嵌时有安全降级路径 |
| SPIKE-WEB-04 | Apple Music 登录和播放兼容性？ | 仅使用已获用户授权的正常服务尝试播放/切换/后台音频 | 通过则实验性启用；不通过仅外部启动 |
| SPIKE-PKG-01 | Portable 真实隔离与 WebView2 Runtime 检测？ | 在两个不同可写位置运行；模拟只读目录、缺少 Runtime | 有可解释的依赖提示和安全数据位置 |
| SPIKE-FILE-01 | PDF 提取/转换、图像与 ZIP 安全性及许可？ | 对常用与损坏测试文件跑解析、压缩包穿越测试、并发取消 | 选择稳定且许可合规的库 |
| SPIKE-PERF-01 | 哪些功能必须 worker/Sidecar？ | 基线记录启动时间、内存、CPU、单页/五页 WebView 指标 | 冻结 NFR 阈值，决定按需进程拆分 |
| SPIKE-ADOBE-01 | 可否安全读取 Adobe 工程结构？ | 非生产样例工程，研究 API/格式、兼容矩阵，严禁写入原件 | 独立立项，V0.1 不阻塞发布 |
| SPIKE-CI-01 | GitHub Windows runner 能否正确打包测试？ | Windows x64 build，生成 NSIS/MSI/Portable ZIP，启动冒烟 | 未完成不得发布可用版本声明 |

SPIKE 报告模板：`目的 / 假设 / 平台版本 / 实现方法 / 测试样例 / 性能数据 / 失败日志 / 结论 / ADR / 负责人 / 日期`。不能用“应该可行”代替实验结果。

## 13. 开发里程碑与 Codex 可执行任务

| 阶段 | 工作包 | 完成条件 |
|---|---|---|
| M0 Foundation | 仓库、Tauri/React/Rust 启动、Rust/TS 类型契约、配置/数据库迁移、CI；进行 Browser/Portable/Perf SPIKE | Windows 11 能启动，关键架构风险有报告和 Gate |
| M1 App Core | 应用扫描/手动添加/收藏/搜索/启动/指定程序打开、错误/日志/持久化 | APP-001~APP-008 可逐项演示和自动化测试 |
| M2 Browser | WebView2 标签与 Profile、Web Apps、收藏/历史/下载、外部回退 | WEB-001~WEB-009 完成，10 页切换稳定性达标 |
| M3 Files | 目录浏览/安全文件操作、后台索引、FTS5、图片/PDF/ZIP 基础转换 | FILE-001~FILE-009 完成，危险输入测试通过 |
| M4 Integration | 记账/摄影自有 Web Apps 注册、Adobe 应用/工程关联、普通备份恢复、Portable 数据迁移 | 联动只使用受支持能力；数据与安全回归通过 |
| M5 Preview Release | 性能回归、日志与错误修复、Windows 10 合规配置兼容测试、安装/卸载/升级、签名/资产校验、GitHub Release | 产品验收清单全部 P0 通过，生成变更日志和已知限制 |

**建议任务粒度**：每个 Codex Cloud 任务仅包含一个可独立验收的模块小功能，尽量 1 个 PR 完成；跨模块接口先提交 Contract PR，再提交实现 PR，最后提交集成 PR。不得让单个任务同时改动应用管理、浏览器内核、文件搜索与打包配置。

## 14. Codex Cloud 工作规范

### 14.1 任务启动时必须执行

1. 阅读 `TOOLBOX_SPEC.md` 中本次涉及的需求 ID；若存在 `AGENTS.md`，先按仓库说明加载。
2. 检查现有模块契约与测试，说明计划修改路径、风险、测试策略。
3. 任务中明确列出 P0 / P1 / SPIKE；**不得自动实施未被要求的 Backlog**。
4. 如依赖 Windows API、WebView2 真实进程或安装包，则标明“云端无法完成的验证”并建立 Windows 验证步骤。
5. 如果 Skills 被要求使用，仅在仓库可访问的情况下读取对应 `.agents/skills/<name>/SKILL.md`；若不可用，记录缺失并继续依照仓库规范完成可行任务，不假称调用成功。

### 14.2 代码约束

- 不为了通过测试把真实操作替换成占位、Mock 或无实际效果按钮；Mock 仅允许在明确命名的单元测试/夹具里。
- 不使用大型依赖只实现微小功能；新增二进制依赖必须说明包体、运行内存、许可与替代方案。
- Tauri 能力白名单最小化，不在配置中为所有远程页面开启 invoke 或全盘文件访问。
- 每个任务都需要最小单元测试；文件操作必须包含错误、权限不足与特殊字符路径。
- 所有路径、进程启动、用户输入使用结构化 API，避免 Shell 字符串拼接。
- 新模块的初始化不得放到启动关键路径，除非需求明确要求且有性能测试依据。
- 配置、数据库 Schema、命令契约发生变化时同步更新文档及迁移测试。

### 14.3 PR 输出模板

每个 PR 描述至少包含：

```text
需求 ID：
功能变化：
影响模块：
权限/安全变化：
数据迁移：
包体与性能影响：
已运行的测试与结果：
无法在 Codex Cloud 验证的 Windows 测试：
已知限制/回滚方式：
```

- 云端 CI 通过 ≠ Windows 验收通过。
- 执行失败不能标“已完成”；必须给出可复现步骤及原因。
- 每次修改 README 或技术规范均需记录对应 ADR / 需求 ID。

## 15. 测试与验收标准

### 15.1 测试矩阵

| 级别 | 内容 | 必须运行的位置 |
|---|---|---|
| Unit | Rust 领域/文件路径/数据迁移/DTO 校验，TS 业务逻辑 | Codex Cloud 或本地 |
| Contract | 命令、事件、Module Manifest、权限拒绝案例 | Codex Cloud + CI |
| Integration | SQLite、索引、转换器、文件操作测试夹具 | CI / Windows（平台相关部分） |
| Windows Browser E2E | 多 WebView、Profile、下载、历史、网页导航和重开恢复 | Windows 11 本地/Windows runner |
| Windows Packaging | NSIS/MSI、Portable ZIP、WebView2 Runtime 检测、卸载 | Windows CI + 本地冒烟 |
| Compatibility | Windows 10 ESU/LTSC 合规环境和 Windows 11 x64 | 有真实或虚拟目标环境时实测 |
| Security | T3 网站阻断 native invoke、路径穿越、ZIP 炸弹、权限撤销 | CI + Windows 本地 |
| Performance | 冷/热启动、私有内存、空闲 CPU、五标签压力、索引/转换压力 | Windows 11 固定基线机 |

### 15.2 发布 V0.1 的强制验收清单

- [ ] AC-001：Windows 11 x64 成功启动、关闭、重开，用户数据保留；不能因为无网络而无法进入工作台。
- [ ] AC-002：目标 Windows 10 合规配置已完成实机/VM 验证；若未能验证，不得声称已兼容。
- [ ] AC-003：至少扫描并启动一个本机真实程序、手动注册一个 EXE、收藏并用指定应用打开文件。
- [ ] AC-004：两个不同浏览器 Profile 的 Cookie/历史隔离得到可复现证据。
- [ ] AC-005：至少 5 个真实网页标签能打开、切换、关闭，历史/收藏/下载可持久化；100 次重复操作无主程序崩溃。
- [ ] AC-006：记账与摄影 Web Apps 支持填写真实 URL 注册及外部打开降级；不得要求公开其私有凭据。
- [ ] AC-007：文件复制、移动、重命名、回收站删除在测试夹具中通过；原文件受转换另存规则保护。
- [ ] AC-008：索引测试目录，文件名/元数据以及 Markdown/可提取文本 PDF 全文查询正确；索引可停止/恢复。
- [ ] AC-009：PNG/JPEG/WebP、基本 PDF 合并拆分、ZIP 创建解压可以真实执行并处理损坏输入。
- [ ] AC-010：恶意路径、损坏压缩包与网页 native API 调用被阻止；无明文凭据泄漏到日志/普通备份。
- [ ] AC-011：SQLite 升级、普通备份/恢复、异常中断后重新打开正常；基本数据恢复演练成功。
- [ ] AC-012：安装包和 Portable ZIP 都可在干净测试机运行；Portable 在可写目录保持数据独立，Runtime 缺失给出明确处理提示。
- [ ] AC-013：没有可选模块也能启动核心程序；不预启动 FFmpeg/Adobe/AI；有性能报告及对目标的达成结果。
- [ ] AC-014：GitHub Actions 出具测试和构建报告，Releases 发布包含 SHA-256 校验、发行说明和当前已知问题；更新不静默安装。
- [ ] AC-015：Apple Music/Adobe 深层解析/浏览器扩展等尚未验证能力标记为实验或未支持，不能混入已完成清单。

### 15.3 缺陷等级和阻断条件

- `Blocker`：数据丢失、未授权本地执行、远程网页越权、安装包打不开、可复现主程序启动崩溃；必须修复。
- `Critical`：重复性索引损坏、跨 Profile 数据串用、主要文件操作损坏源文件、多标签频繁崩溃；必须修复。
- `Major`：重要 P0 功能不能工作、恢复不可用、明显资源泄漏；必须修复或与产品所有者确认降级及显式已知问题。
- `Minor`：不影响 P0 的界面细节/非核心扩展问题；可以记录并排期。

## 16. 发行与维护策略

### 16.1 交付物

- 安装版：Windows x64 NSIS `.exe` 和/或 WiX `.msi`，应验证卸载不默认删除个人数据。
- Portable：`Toolbox-v0.1-windows-x64-portable.zip`，内含 `app/`、`data/` 目录规划及许可；不是单文件 EXE。
- 每个 Release：版本号、Git commit、支持系统、已知问题、变更日志、SHA-256 校验与依赖/许可清单。
- 安装版更新：在具备可靠签名/可信更新源后支持用户确认安装；V0.1 未达到验收条件时可先做版本提示和手动下载。Portable 手动替换**程序目录**，不能覆盖 `data/`。

### 16.2 兼容与维护

- 以测试矩阵报告真实支持的 OS 版本；Windows 10 未获得支持的普通系统不作为安全兼容承诺。
- 构建依赖及 WebView2 API 版本明确锁定；跟进 Windows/WebView2 更新回归。
- 程序退出不会删除用户原文件或已存在的外部软件工程。
- 未来公开发行必须确认开源许可证、依赖许可证和第三方品牌/软件接口合规要求。

## 17. 后续独立 ADR（不在本轮拍板）

以下项目**已有需求意向但还没有足够的前提条件**：

1. 手机端采用原生、Tauri Mobile 或独立 Web/移动应用；不要在 V0.1 建空的远控服务器。
2. 手机远程操作电脑如何鉴权、配对、端到端加密及离线状态处理。
3. 云同步采用何种服务、冲突解决与加密策略。
4. Photoshop/Premiere/After Effects 使用独立解析器还是官方插件；具体宿主版本兼容矩阵。
5. 插件和 Skills 的代码执行沙箱与签名发布策略。
6. 本地 AI 模型的设备检测、模型下载/管理和显存预算。
7. Apple Music 的网页播放是否兼容 WebView2；若未来要用 MusicKit，必须重新评估开发者资格与授权。
8. 是否需要把所有媒体转换封装成独立 `media-pack`，以及编解码器许可证与更新方式。
9. Windows 10 的最低具体发行版/ESU 及 WebView2 版本矩阵，需在目标设备信息确认后冻结。

## 18. 参考资料与版本说明

核实日期：2026-10-14。以下官方资料只用于说明技术边界；编码前仍需查阅与锁定依赖版本对应的最新文档。

1. [Tauri 2 Webview API — 多子 WebView 创建需 `unstable` 特性](https://v2.tauri.app/reference/javascript/api/namespacewebview/)
2. [Tauri Windows Installer 与 WebView2 安装选项](https://v2.tauri.app/distribute/windows-installer/)
3. [Tauri Windows WebView2 版本说明](https://v2.tauri.app/reference/webview-versions/)
4. [Microsoft WebView2 Multiple Profiles](https://learn.microsoft.com/en-in/microsoft-edge/webview2/concepts/multi-profile-support)
5. [Microsoft WebView2 User Data Folder](https://github.com/MicrosoftDocs/edge-developer/blob/main/microsoft-edge/webview2/concepts/user-data-folder.md)
6. [Microsoft Edge 与 WebView2 的功能差异](https://github.com/MicrosoftDocs/edge-developer/blob/main/microsoft-edge/webview2/concepts/browser-features.md)
7. [Microsoft Windows 10 生命周期与 ESU](https://support.microsoft.com/en-us/windows/deployment/updates-lifecycle/windows-10-support-has-ended-on-october-14-2025)
8. [Apple Music API Developer Token](https://developer.apple.com/documentation/AppleMusicAPI/generating-developer-tokens)
9. [Apple Music MusicKit 用户授权](https://developer.apple.com/documentation/applemusicapi/user-authentication-for-musickit)
10. [Adobe Photoshop UXP Scripting](https://developer.adobe.com/photoshop/uxp/scripting/)
11. [Adobe Premiere Pro UXP API](https://developer.adobe.com/premiere-pro/uxp/uxp-api/)
12. [Adobe After Effects UXP API](https://developer.adobe.com/after-effects/uxp/after-effects-api/)

---

## 19. 项目所有者的后续检查项

- [ ] 将本文件提交到 GitHub 仓库根目录，作为 V0.1 需求基线。
- [ ] 新建 `AGENTS.md`，将第 14 节工作要求转成 Codex 必须遵循的指令。
- [ ] 在 Windows 11 开发电脑安装并检查 Rust、Node.js、WebView2 Runtime、Tauri 构建先决条件。
- [ ] 使用最小原型优先完成 `SPIKE-WEB-01 / SPIKE-PERF-01 / SPIKE-PKG-01`。
- [ ] 记录目标 Windows 10 安全支持配置和兼容测试策略。
- [ ] 当记账、摄影 Web Apps URL 已确定时，增加本地配置；不要在公开仓库硬编码私人地址或密钥。
- [ ] 等 V0.1 稳定运行再扩展 V0.2，避免大型依赖引起启动和内存回归。

**最终约束**：任何开发者（包含 Codex）不得将所有候选功能打包进单个 EXE，不得以牺牲资源占用、稳定性、用户文件安全来提前实现后续功能；V0.1 以三个核心模块的实际可用性、可验证性和可维护性为最高优先级。
