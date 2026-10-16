# SPIKE-PERF-01 — 完整进程树基线

需求：SPIKE-PERF-01、ADR-014；规范 §9；WF-02。2026-10-15 最终版本完整基线已实测，所述口径下预算 PASS；初轮热启动 FAIL 原始数据保留，目标未调整。

WF-01 的重复启动与早期内存证据保持在 `docs/performance/`，不将它们改称冷启动或完整 CPU 验收。本任务在原生实验产物通过后执行 `tools/spikes/windows-gate-a.ps1`，不加 `-SkipPerformance`。

采样口径：

- 20 次 process-cold：每次新进程、全新 UDF；**不清空 OS 文件缓存，不等于系统重启后的冷启动**。
- warm：先做一次不计入样本的初始化，然后同 UDF 正常关闭重开 20 次。
- 保存 launcher 等待时间及 Rust 入口到 React 双帧后健康 IPC 就绪时间。后者排除 OS 加载 EXE，不能混用。P90 使用排序后 nearest-rank 第 18 个样本。
- Shell-only、1 标签、5 标签都采集宿主与所有后代 WebView2 进程的 PrivateBytes、CPU 秒数；不只看 Rust EXE。
- 空闲 CPU 取约 60 秒累计 CPU 差 / 实际经过秒 / 逻辑处理器数。前后进程集合变化或累计 CPU 倒退时记为 NOT VERIFIED 并重测，不能算成 0 或负值通过。
- 报告 OS、CPU、物理内存、WebView2 Runtime 和产物来源。资源组使用共享环境命名 Profile 方案；0 标签不创建外部网页。

环境：Windows 11 家庭版中文版 build 26200；Ultra 9 185H、22 逻辑处理器；物理内存 33,730,662,400 bytes；WebView2 155.0.4283.45；测试所在 C 盘在运行后核实为 NVMe SSD。使用显式实验 feature EXE 8,543,744 bytes；相对先前 WF-01 产物增加 239,616 bytes，不是同提交生产构建的体积对比，也不能全部归因于单个依赖。完整来源与 SHA 见 [provenance](../performance/wf02-provenance.json)。

| 项目 | 实际结果 | 目标与结论 |
|---|---|---|
| 20 次新进程/新 UDF，launcher P90 | 907.4532 ms | 该口径 ≤3000 ms；OS-cache-cold NOT RUN |
| 20 次 warm reopen，launcher P90 | 791.3158 ms | ≤2000 ms，PASS |
| 同两组 Rust→前端 P90 | 833 / 728 ms | 辅助分解，不替代 launcher 结果 |
| Shell-only 私有内存 / 60秒空闲CPU | 190,373,888 bytes / 0.01639% | 无标签内存目标 ≤300 MB、CPU <2%，PASS |
| 1 标签私有内存 / 空闲CPU | 226,496,512 bytes / 0.02340% | 已记录；CPU <2% |
| 5 标签私有内存 / 空闲CPU | 360,361,984 bytes / 0.04206% | 已记录；300 MB 是无标签 Shell 目标，不能套用到5页后掩盖数据 |

三个资源组前后 PID 集合均稳定，区间分别约 60.65 / 60.69 / 60.78 秒。资源是完整进程树，不是仅 Rust EXE。原始 40 启动样本和各进程明细见 [JSON](../performance/wf02-local-baseline.json)。

目标保持：冷启动 P90 ≤3 秒、热启动 P90 ≤2 秒、无标签 Shell 私有内存目标 ≤300 MB、空闲 CPU <2%。初轮提交9790435的冷/热P90为2595.9806/2158.7017ms，热启动FAIL，数据及来源完整保留在 [initial baseline](../performance/wf02-initial-baseline.json) / [initial provenance](../performance/wf02-initial-provenance.json)。最终提交d4ca638同机同脚本完整复测达标；没有证据把两轮差异归因于SDK预检或某项改动，也不删除较慢结果。后续需保持同口径回归，超过冻结基线15%按规范分析；WF-01早期repeated-launch口径不同，不能直接计算回归率。真实网页复杂内容、索引/转换/Sidecar、长期泄漏尚未测试。

当前没有证据要求增加常驻 Sidecar；保持轻量 Shell、浏览器按需创建。保留启动波动和未执行范围，不放宽阈值；是否接受此宿主作为继续实现的基线由 Gate A 审阅确定，不表示已达到所有发行性能/稳定性标准。
