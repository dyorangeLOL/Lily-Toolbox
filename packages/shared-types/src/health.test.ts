import { describe, expect, it } from "vitest";
import fixture from "../fixtures/health.json";
import { parseHealth } from "./health";

describe("native health contract", () => {
  it("accepts the same fixture consumed by Rust", () => {
    expect(parseHealth(fixture)).toEqual(fixture);
  });
  it.each([
    null,
    [],
    {},
    { ...fixture, api_version: 2 },
    { ...fixture, token: "secret" },
    { ...fixture, runtime: "browser" },
  ])("rejects malformed or untrusted health payload %j", (value) =>
    expect(() => parseHealth(value)).toThrow(),
  );
});
