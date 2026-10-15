# SPIKE-PERF-01 — 完整进程树基线

需求：SPIKE-PERF-01、ADR-014；规范 §9；WF-02。状态：in-progress；完整基线 NOT RUN。

WF-01 的重复启动与早期内存证据保持在 `docs/performance/`，不将它们改称冷启动或完整 CPU 验收。本任务在原生实验产物通过后执行 `tools/spikes/windows-gate-a.ps1`，不加 `-SkipPerformance`。

采样口径：

- 20 次 process-cold：每次新进程、全新 UDF；**不清空 OS 文件缓存，不等于系统重启后的冷启动**。
- warm：先做一次不计入样本的初始化，然后同 UDF 正常关闭重开 20 次。
- 保存 launcher 等待时间及 Rust 入口到 React 双帧后健康 IPC 就绪时间。后者排除 OS 加载 EXE，不能混用。P90 使用排序后 nearest-rank 第 18 个样本。
- Shell-only、1 标签、5 标签都采集宿主与所有后代 WebView2 进程的 PrivateBytes、CPU 秒数；不只看 Rust EXE。
- 空闲 CPU 取约 60 秒累计 CPU 差 / 实际经过秒 / 逻辑处理器数。前后进程集合变化或累计 CPU 倒退时记为 NOT VERIFIED 并重测，不能算成 0 或负值通过。
- 报告 OS、CPU、物理内存、WebView2 Runtime 和产物来源。资源组使用共享环境命名 Profile 方案；0 标签不创建外部网页。

目标保持：冷启动 P90 ≤3 秒、热启动 P90 ≤2 秒、无标签 Shell 私有内存目标 ≤300 MB、空闲 CPU <2%。当前无新增实际数值，不能宣称达标。真实网页复杂内容、索引/转换/Sidecar、长期泄漏尚未测试。

当前没有证据要求增加常驻 Sidecar；保持轻量 Shell、浏览器按需创建。后续若采样超预算，先报告差距和可复现原因，再决定拆分，不能为了数字删掉子进程或改变口径。
