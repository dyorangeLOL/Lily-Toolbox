import { describe, it, expect } from "vitest";
import fixture from "../fixtures/core.json";
import {
  parseRequest,
  parseResponse,
  parseEvent,
  parseManifest,
  parseModuleList,
} from "./core";

describe("WF-03 shared Rust/TS contracts", () => {
  it.each(fixture.requests)("roundtrips domain request $params.command", (v) =>
    expect(parseRequest(v)).toEqual(v),
  );
  it.each(fixture.invalid_requests)("rejects invalid request %#", (v) =>
    expect(() => parseRequest(v)).toThrow(),
  );
  it.each(fixture.responses)("accepts response %#", (v) =>
    expect(parseResponse(v)).toEqual(v),
  );
  it.each(fixture.invalid_responses)("rejects contradictory response %#", (v) =>
    expect(() => parseResponse(v)).toThrow(),
  );
  it.each(fixture.events)("accepts typed event %#", (v) =>
    expect(parseEvent(v)).toEqual(v),
  );
  it("rejects wrong source, excess progress and sensitive extra fields", () => {
    const e = fixture.events[0]!;
    expect(() => parseEvent({ ...e, source_module: "untrusted" })).toThrow();
    expect(() =>
      parseEvent({
        ...e,
        payload: {
          topic: "convert.jobChanged",
          data: { job_id: "j", state: "running", progress_percent: 101 },
        },
      }),
    ).toThrow();
    expect(() => parseEvent({ ...e, token: "secret" })).toThrow();
  });
  it("validates builtin manifest and rejects executable/third-party injection", () => {
    const m = fixture.manifests[0]!;
    expect(parseManifest(m)).toEqual(m);
    expect(parseModuleList([m])).toEqual([m]);
    for (const patch of [
      { entry: "C:\\evil.exe" },
      { kind: "optional" },
      { permissions: ["execute_shell"] },
      { version: "01.0.0" },
      { dependencies: [{ id: "x", version: "*" }] },
    ])
      expect(() => parseManifest({ ...m, ...patch })).toThrow();
  });
});
