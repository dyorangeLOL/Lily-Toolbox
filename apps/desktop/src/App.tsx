import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseHealth, type HealthStatus } from "@toolbox/shared-types";

export function App() {
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const check = useCallback(async (signal?: AbortSignal) => {
    setPending(true);
    setError(null);
    try {
      const response = parseHealth(await invoke<unknown>("health_check"));
      if (signal?.aborted) return;
      setHealth(response);
      await new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      });
      if (signal?.aborted) return;
      await invoke("frontend_ready");
    } catch {
      if (signal?.aborted) return;
      setHealth(null);
      setError("无法连接桌面宿主。请在 Toolbox 桌面程序中重试健康检查。");
    } finally {
      if (!signal?.aborted) setPending(false);
    }
  }, []);
  useEffect(() => {
    const controller = new AbortController();
    void check(controller.signal);
    return () => controller.abort();
  }, [check]);
  return (
    <main>
      <h1>Toolbox</h1>
      <p>WF-01 最小桌面骨架，业务功能尚未实现。</p>
      <button type="button" disabled={pending} onClick={() => void check()}>
        {pending ? "检查中…" : "检查桌面宿主"}
      </button>
      <p role="status" aria-live="polite">
        {health
          ? `已连接 Rust 宿主，版本 ${health.version}，协议 ${health.api_version}`
          : "尚未连接"}
      </p>
      {error && <p role="alert">{error}</p>}
    </main>
  );
}
