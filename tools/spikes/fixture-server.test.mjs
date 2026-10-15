import { test } from "node:test";
import assert from "node:assert/strict";
import { startFixtureServer, validateEvents } from "./fixture-server.mjs";

test("fixture returns real HTML and download bytes over loopback HTTP", async () => {
  const { server, port } = await startFixtureServer();
  try {
    const html = await (await fetch(`http://127.0.0.1:${port}/page`)).text();
    assert.ok(html.includes("document.cookie"));
    assert.equal(await (await fetch(`http://127.0.0.1:${port}/download`)).text(), "TOOLBOX-DOWNLOAD-FIXTURE");
  } finally { await new Promise(resolve => server.close(resolve)); }
});

const event = { slot: 0, profile: "A", before: "", after: "wf02_profile=A", nativeDenied: true, width: 760, height: 350 };
test("verifier rejects cross-profile cookies and remote native privilege", () => {
  assert.throws(() => validateEvents([{ ...event, after: "wf02_profile=B" }], 1));
  assert.throws(() => validateEvents([{ ...event, nativeDenied: false }], 1));
});
test("verifier separates fresh-root and persistence requirements", () => {
  assert.doesNotThrow(() => validateEvents([event], 1, { fresh: true }));
  assert.throws(() => validateEvents([event], 1, { persistent: true }));
  assert.throws(() => validateEvents([{ ...event, before: "wf02_profile=A" }], 1, { fresh: true }));
});
