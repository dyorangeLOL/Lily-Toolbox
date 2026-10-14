# Codex 任务进度

基线：[TOOLBOX_SPEC.md](../../TOOLBOX_SPEC.md) v1.0、[CODEX_WORKFLOW.md](../../CODEX_WORKFLOW.md) v1.0；规则：[AGENTS.md](../../AGENTS.md)；[待决事项](DECISIONS_PENDING.md)。

最新更新：2026-10-14（Asia/Shanghai）。状态词：not-started / in-progress / awaiting-windows-validation / verified / blocked / deferred。verified 仅指对应任务，不代表 V0.1 产品已完成、PR 已合并或所有者已批准。

## 仓库审查与本次范围

- 基线 commit：[3ecce7f](https://github.com/dyorangeLOL/Lily-Toolbox/commit/3ecce7fa349f4f7393550ea914149e496ffd46ca)。实际树只有 CODEX_WORKFLOW.md 和 TOOLBOX_SPEC.md。
- 不存在 AGENTS.md、README、源码、CI、lockfile、package.json 或 Cargo workspace；没有已有内容需要合并/覆盖。两份基线保持原样。
- 当前任务仅 WF-00：四份治理文档。P0 业务功能没有实现；P1/P2 没有启动；SPIKE 没有执行；没有 UI、依赖或应用骨架。
- 需求关联：ADR-002/011/012/014；规范 §13–15。本次不新建或修改架构、命令/事件契约、数据库 Schema，也不决定浏览器宿主。

## 任务台账

后续行的“—”表示尚无 PR/commit 或验证人；NOT RUN 表示尚未执行，不能计为通过。映射取自对应 WF 的需求行；映射不等于实现。

| 任务 | 状态 | 需求映射 | 前置 / Gate | PR / commit | 验证人 | 云端测试 | Windows 测试 | 阻塞 / 最新更新 |
|---|---|---|---|---|---|---|---|---|
| WF-00 | verified | ADR-002/011/012/014；§13–15 | 两份基线已提交 | [PR #1](https://github.com/dyorangeLOL/Lily-Toolbox/pull/1)；[治理提交 997cb16](https://github.com/dyorangeLOL/Lily-Toolbox/commit/997cb1656f9f5da15d18f8b0e6d87de789024e9d) | Codex（本地检查与自审） | NOT RUN：未在 Codex Cloud/CI 运行 | NOT RUN：无应用；文档检查见下文 | 文档检查与 Codex 自审 PASS；外部审阅待完成；2026-10-14 |
| WF-01 | not-started | APP-007；ADR-003/011/012 | WF-00 verified 与审查合并 | — | — | NOT RUN | NOT RUN | 未派发；2026-10-14 |
| WF-02 | not-started | WEB-010/007；SPIKE-PKG-01/PERF-01；§12 | WF-01 Windows 可启动 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-03 | not-started | ADR-009/010/011；§6–8 | Gate A | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-04 | not-started | APP-001/002/003（部分） | WF-03 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-05 | not-started | APP-004/005/006/007；INT-001 | WF-04 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-06 | not-started | WEB-001/002/008/009 | Gate A + WF-03；宿主 ADR 批准 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-07 | not-started | WEB-003/004/007；§4 | WF-06 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-08 | not-started | WEB-005/006/008；INT-004；WEB-011（SPIKE） | WF-07 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-09 | not-started | FILE-001/002；INT-005；ADR-008 | WF-03 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-10 | not-started | FILE-003/004/005/006 | WF-09 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-11 | not-started | FILE-007/008/009；§5 | WF-09 + 依赖库验证 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-12 | not-started | INT-001…005；INT-007（独立 SPIKE） | WF-05 + WF-08 + WF-09 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-13 | not-started | §6；AC-011 | WF-03 + 必要业务表已实现 | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-14 | not-started | §8–9、§15 | 所有 P0 业务完成；Gate B | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-15 | not-started | APP-008；ADR-004/013；§16 | Gate B；Gate C | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |
| WF-16 | not-started | §15–16；AC-001…AC-015 | Gate C + 所有者审批；Gate D | — | — | NOT RUN | NOT RUN | 未到阶段；2026-10-14 |

## WF-00 验证证据

执行环境：本地 Windows PowerShell；不是 Codex Cloud 或 GitHub Actions。检查人：Codex。以下检查只验证治理文档，不验证应用功能。2026-10-14 已实际执行；输出见下方。

| 检查 | 结果 | 证据 / 限制 |
|---|---|---|
| 基线仓库树审查 | PASS | git ls-tree -r --name-only 3ecce7f：仅两份基线 |
| 四份输出存在、AGENTS ≤120 行、相对文档链接有效、WF-00…16 各一行且后续未开始 | PASS | 实际输出：files=4；AGENTS lines=69；local links=16；WF rows=17；later tasks=not-started |
| 改动仅四份输出；两份基线未改 | PASS | scope=4；baselines=unchanged；新增仅四份要求的文档 |
| 格式/空白检查 | PASS | git diff --cached --check 返回 0，无空白错误 |
| Code Review | PASS（Codex 自审） | 2026-10-14 对照 WF-00 六项要求及规范 §14，无阻断项；外部 PR 审阅 NOT RUN，不冒充所有者批准 |
| TS/Rust 构建、类型、单元与集成测试 | NOT RUN | 无源码、工作区、依赖或测试脚本；本次为文档验收，不创建空测试项目 |
| Codex Cloud / GitHub Actions | NOT RUN | 本次在本地执行，仓库没有 CI |
| Windows 应用启动、WebView2/Profile、打包/Portable、Windows 10、性能 | NOT RUN | 没有程序或产物，属于后续 WF；当前无操作步骤可执行，不声明通过 |

### 可复现的文档检查

在仓库根目录 PowerShell 运行以下代码。检查失败会抛出错误；不要将输出视为 Windows 功能验收。

```powershell
$ErrorActionPreference = 'Stop'
$outputs = @('AGENTS.md', 'README.md', 'docs/progress/CODEX_PROGRESS.md', 'docs/progress/DECISIONS_PENDING.md')
foreach ($file in $outputs) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Missing: $file" }
}
$agentLines = @(Get-Content -LiteralPath 'AGENTS.md' -Encoding UTF8).Count
if ($agentLines -gt 120) { throw "AGENTS exceeds 120 lines: $agentLines" }
$linkCount = 0
foreach ($file in $outputs) {
    $body = [IO.File]::ReadAllText((Join-Path (Get-Location) $file))
    foreach ($match in [regex]::Matches($body, '\[[^\]]+\]\(([^)]+)\)')) {
        $target = $match.Groups[1].Value
        if ($target -match '^https?://') { continue }
        $resolved = Join-Path (Split-Path -Parent (Join-Path (Get-Location) $file)) $target
        if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) { throw "Broken link: $file -> $target" }
        $linkCount++
    }
}
$progress = [IO.File]::ReadAllText((Join-Path (Get-Location) 'docs/progress/CODEX_PROGRESS.md'))
$rows = [regex]::Matches($progress, '(?m)^\| WF-(\d{2}) \| ([a-z-]+) \|')
if ($rows.Count -ne 17) { throw "Expected 17 task rows: $($rows.Count)" }
for ($i = 0; $i -lt 17; $i++) {
    if ([int]$rows[$i].Groups[1].Value -ne $i) { throw "Missing or duplicate WF: $i" }
    if ($i -gt 0 -and $rows[$i].Groups[2].Value -ne 'not-started') { throw "Later WF started: $i" }
}
$changed = @(git diff --cached --name-only '3ecce7fa349f4f7393550ea914149e496ffd46ca')
if ($LASTEXITCODE -ne 0) { throw 'git diff failed' }
$scopeDiff = @(Compare-Object ($outputs | Sort-Object) ($changed | Sort-Object))
if ($scopeDiff.Count -ne 0) { throw 'Staged scope must contain exactly four WF-00 outputs' }
git diff --cached --exit-code '3ecce7fa349f4f7393550ea914149e496ffd46ca' -- TOOLBOX_SPEC.md CODEX_WORKFLOW.md
if ($LASTEXITCODE -ne 0) { throw 'Baseline changed' }
git diff --cached --check '3ecce7fa349f4f7393550ea914149e496ffd46ca'
if ($LASTEXITCODE -ne 0) { throw 'Whitespace check failed' }
"PASS: files=4; AGENTS lines=$agentLines; local links=$linkCount; WF rows=$($rows.Count); later tasks=not-started; scope=4; baselines=unchanged; whitespace=clean"
```

检查针对当前暂存区相对基线的差异：提交前可先用 `git add -- AGENTS.md README.md docs/progress/CODEX_PROGRESS.md docs/progress/DECISIONS_PENDING.md`。提交后在本 PR 分支干净检出中也可直接运行；比较仍以固定基线为准。不要重置已有工作。

## 审查、影响与下一步

- 审查方式：逐项对照工作流 WF-00 的六项实施要求、验收和规范 §14；结论 PASS：规则覆盖范围、真实性、权限、原件、懒加载和 PR 格式；台账完整且无后续完成项；待决事项只含产品 Gate；README 不承诺未实现功能。自审人 Codex，日期 2026-10-14；没有独立审查人参与。产品所有者审阅/合并待完成，由 PR 表示。
- 权限、数据迁移、运行包体与启动/内存：本次仅文档，没有运行时变化；性能测试 NOT RUN，不宣称达标。
- 回滚：关闭未合并 PR；若已合并，revert 本次治理文档提交，不修改需求基线或用户原件。
- 下一可派发任务：WF-01，仅在 WF-00 验收、审查及合并后另行授权；本次结束于 WF-00。
