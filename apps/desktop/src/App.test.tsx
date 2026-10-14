import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { App } from "./App";

// Unit-test fixture only; production always invokes the native host.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const native = vi.mocked(invoke);

describe("minimal native health interaction", () => {
  beforeEach(() => native.mockReset());
  it("acknowledges readiness only after a valid native response", async () => {
    native
      .mockResolvedValueOnce({
        api_version: 1,
        version: "0.1.0",
        runtime: "tauri-rust",
      })
      .mockResolvedValueOnce(undefined);
    render(<App />);
    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent("已连接 Rust 宿主"),
    );
    await waitFor(() =>
      expect(native.mock.calls.map(([command]) => command)).toEqual([
        "health_check",
        "frontend_ready",
      ]),
    );
  });
  it("shows failure without fabricated readiness", async () => {
    native.mockRejectedValueOnce(new Error("IPC unavailable"));
    render(<App />);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "无法连接桌面宿主",
    );
    expect(native).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("status")).toHaveTextContent("尚未连接");
  });
  it("rejects an unsupported native protocol", async () => {
    native.mockResolvedValueOnce({
      api_version: 2,
      version: "0.1.0",
      runtime: "tauri-rust",
    });
    render(<App />);
    await screen.findByRole("alert");
    expect(native).toHaveBeenCalledTimes(1);
  });
});
