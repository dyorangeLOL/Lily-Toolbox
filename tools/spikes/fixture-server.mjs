import http from "node:http";
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

// Controlled localhost test fixture, not a fake browser or production service.
export function startFixtureServer() {
  const events = [];
  const server = http.createServer(async (request, response) => {
    const url = new URL(request.url, "http://127.0.0.1");
    if (url.pathname === "/page") {
      response.writeHead(200, { "Content-Type": "text/html; charset=utf-8", "Cache-Control": "no-store" });
      response.end(`<!doctype html><html><head><meta charset="utf-8"><title>WF-02 controlled fixture</title></head><body><h1>Real WebView2 fixture</h1><a id="download" href="/download">Download fixture</a><script>
const params = new URL(location.href).searchParams;
const profile = params.get('profile');
const before = document.cookie;
if (!document.cookie.split(';').some(part => part.trim().startsWith('wf02_profile='))) {
  document.cookie = 'wf02_profile=' + profile + '; Path=/; Max-Age=3600; SameSite=Lax';
}
(async () => {
  let denied = true;
  if (window.__TAURI_INTERNALS__?.invoke) {
    for (const [command,args] of [['health_check',{}],['core_dispatch',{request:{request_id:'remote-1',api_version:1,caller_context:{module_id:'toolbox.shell'},params:{command:'modules.list',args:{}}}}]]) {
      try { await window.__TAURI_INTERNALS__.invoke(command,args); denied = false; } catch { /* denied by native ACL */ }
    }
  }
  await fetch('/seen', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({run:params.get('run'),slot:Number(params.get('slot')),profile,before,after:document.cookie,nativeDenied:denied,width:innerWidth,height:innerHeight})});
})();
</script></body></html>`);
    } else if (url.pathname === "/seen" && request.method === "POST") {
      let body = "";
      for await (const chunk of request) {
        body += chunk;
        if (body.length > 8192) { response.writeHead(413); response.end(); return; }
      }
      try {
        const value = JSON.parse(body);
        if (!/^[a-zA-Z0-9-]{1,64}$/.test(value.run) || !["A", "B"].includes(value.profile) || !Number.isInteger(value.slot) || value.slot < 0 || value.slot > 9) throw new Error("Invalid fixture event");
        events.push(value);
        response.writeHead(204); response.end();
      } catch { response.writeHead(400); response.end(); }
    } else if (url.pathname === "/events") {
      response.writeHead(200, { "Content-Type": "application/json" });
      response.end(JSON.stringify(events.filter(event => event.run === url.searchParams.get("run"))));
    } else if (url.pathname === "/download") {
      response.writeHead(200, { "Content-Type": "application/octet-stream", "Content-Disposition": 'attachment; filename="fixture.txt"' });
      response.end("TOOLBOX-DOWNLOAD-FIXTURE");
    } else { response.writeHead(404); response.end(); }
  });
  return new Promise((resolveReady) => server.listen(0, "127.0.0.1", () => resolveReady({ server, events, port: server.address().port })));
}

export function validateEvents(events, tabs, { fresh = false, persistent = false } = {}) {
  assert.equal(new Set(events.map(event => event.slot)).size, tabs, "Not every real view reached the server");
  const first = new Map();
  const slots = new Set();
  for (const event of events) {
    assert.ok(Number.isInteger(event.slot) && event.slot >= 0 && event.slot < tabs);
    assert.equal(event.profile, event.slot % 2 === 0 ? "A" : "B");
    assert.equal(event.nativeDenied, true, "Remote fixture received native access");
    assert.ok(event.width > 0 && event.height > 0, "No real viewport dimensions");
    assert.ok(event.after.split(";").some(part => part.trim() === `wf02_profile=${event.profile}`), "Cross-profile cookie leakage");
    if (first.has(event.profile) && !slots.has(event.slot)) {
      assert.ok(event.before.includes(`wf02_profile=${event.profile}`), "Same-profile views did not share cookies");
    }
    if (!first.has(event.profile)) first.set(event.profile, event);
    slots.add(event.slot);
  }
  for (const event of first.values()) {
    if (fresh) assert.ok(!event.before.includes("wf02_profile="), "Fresh root reused another root's cookie");
    if (persistent) assert.ok(event.before.includes(`wf02_profile=${event.profile}`), "Cookie did not persist across restart");
  }
  return { observed_views: tabs, observed_documents: events.length, cookie_isolation: true, same_profile_sharing: tabs >= 3, remote_native_denied: true };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const instance = await startFixtureServer();
  writeFileSync(process.argv[2], JSON.stringify({ port: instance.port }));
}
