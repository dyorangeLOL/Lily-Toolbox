export type Command =
  | {
      command:
        | "modules.list"
        | "modules.capabilities"
        | "apps.list"
        | "browser.listProfiles"
        | "convert.capabilities";
      args: Record<string, never>;
    }
  | { command: "files.list" | "backup.verify"; args: { path: string } }
  | { command: "search.query"; args: { query: string } };
export interface CoreRequest {
  request_id: string;
  api_version: 1;
  caller_context: { module_id: "toolbox.shell" };
  params: Command;
}
export interface CoreError {
  code: string;
  message: string;
  retryable: boolean;
}
export type CoreResponse = { request_id: string; api_version: 1 } & (
  { success: true; data: unknown } | { success: false; error: CoreError }
);
export type Capability =
  | "modules_read"
  | "file_read"
  | "file_write"
  | "app_launch"
  | "browser_manage"
  | "convert_run"
  | "backup_restore";
export interface ModuleManifest {
  id: string;
  version: string;
  api_version: 1;
  kind: "builtin";
  features: string[];
  permissions: Capability[];
  dependencies: { id: string; version: string }[];
  entry: string;
  lifecycle:
    "registered" | "available" | "activated" | "deactivated" | "failed";
}
export type JobState =
  "queued" | "running" | "completed" | "failed" | "cancelled";
export interface JobAccepted {
  job_id: string;
}
export interface CoreEvent {
  event_id: string;
  timestamp: number;
  source_module: string;
  schema_version: 1;
  payload:
    | {
        topic: "convert.jobChanged";
        data: { job_id: string; state: JobState; progress_percent: number };
      }
    | {
        topic: "permission.changed";
        data: { grant_id: string; revoked: boolean };
      };
}

function fail(): never {
  throw new Error("Unsupported core API contract");
}
function object(v: unknown): Record<string, unknown> {
  if (typeof v !== "object" || v === null || Array.isArray(v)) fail();
  return v as Record<string, unknown>;
}
function keys(v: Record<string, unknown>, expected: string) {
  if (Object.keys(v).sort().join(",") !== expected.split(",").sort().join(","))
    fail();
}
const id = (v: unknown): v is string =>
  typeof v === "string" && /^[A-Za-z0-9._-]{1,96}$/.test(v);
const text = (v: unknown): v is string => typeof v === "string";

export function parseRequest(value: unknown): CoreRequest {
  const v = object(value);
  keys(v, "request_id,api_version,caller_context,params");
  const c = object(v.caller_context);
  keys(c, "module_id");
  if (
    !id(v.request_id) ||
    v.api_version !== 1 ||
    c.module_id !== "toolbox.shell"
  )
    fail();
  const p = object(v.params);
  keys(p, "command,args");
  const a = object(p.args);
  switch (p.command) {
    case "modules.list":
    case "modules.capabilities":
    case "apps.list":
    case "browser.listProfiles":
    case "convert.capabilities":
      keys(a, "");
      break;
    case "files.list":
    case "backup.verify":
      keys(a, "path");
      if (!text(a.path)) fail();
      break;
    case "search.query":
      keys(a, "query");
      if (!text(a.query)) fail();
      break;
    default:
      fail();
  }
  return value as CoreRequest;
}

export function parseResponse(value: unknown): CoreResponse {
  const v = object(value);
  if (!id(v.request_id) || v.api_version !== 1) fail();
  if (v.success === true) keys(v, "request_id,api_version,success,data");
  else if (v.success === false) {
    keys(v, "request_id,api_version,success,error");
    const e = object(v.error);
    keys(e, "code,message,retryable");
    if (!text(e.code) || !text(e.message) || typeof e.retryable !== "boolean")
      fail();
  } else fail();
  return value as CoreResponse;
}

export function parseEvent(value: unknown): CoreEvent {
  const v = object(value);
  keys(v, "event_id,timestamp,source_module,schema_version,payload");
  if (
    !id(v.event_id) ||
    v.schema_version !== 1 ||
    !Number.isSafeInteger(v.timestamp) ||
    (v.timestamp as number) <= 0
  )
    fail();
  const p = object(v.payload);
  keys(p, "topic,data");
  const d = object(p.data);
  if (p.topic === "convert.jobChanged") {
    keys(d, "job_id,state,progress_percent");
    if (
      v.source_module !== "toolbox.convert" ||
      !id(d.job_id) ||
      !["queued", "running", "completed", "failed", "cancelled"].includes(
        d.state as string,
      ) ||
      !Number.isInteger(d.progress_percent) ||
      (d.progress_percent as number) < 0 ||
      (d.progress_percent as number) > 100
    )
      fail();
  } else if (p.topic === "permission.changed") {
    keys(d, "grant_id,revoked");
    if (
      v.source_module !== "toolbox.modules" ||
      !id(d.grant_id) ||
      typeof d.revoked !== "boolean"
    )
      fail();
  } else fail();
  return value as CoreEvent;
}

const semver =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*))*))?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;
export function parseManifest(value: unknown): ModuleManifest {
  const v = object(value);
  keys(
    v,
    "id,version,api_version,kind,features,permissions,dependencies,entry,lifecycle",
  );
  if (
    !id(v.id) ||
    !v.id.startsWith("toolbox.") ||
    !text(v.version) ||
    !semver.test(v.version) ||
    v.api_version !== 1 ||
    v.kind !== "builtin" ||
    v.entry !== `builtin:${v.id}`
  )
    fail();
  if (
    !Array.isArray(v.features) ||
    v.features.length > 32 ||
    !v.features.every(id)
  )
    fail();
  const caps = [
    "modules_read",
    "file_read",
    "file_write",
    "app_launch",
    "browser_manage",
    "convert_run",
    "backup_restore",
  ];
  if (
    !Array.isArray(v.permissions) ||
    v.permissions.length > 16 ||
    !v.permissions.every((c) => caps.includes(c))
  )
    fail();
  // No externally resolved dependencies or executable entries in V0.1.
  if (
    !Array.isArray(v.dependencies) ||
    v.dependencies.length !== 0 ||
    !["registered", "available", "activated", "deactivated", "failed"].includes(
      v.lifecycle as string,
    )
  )
    fail();
  return value as ModuleManifest;
}

export function parseModuleList(value: unknown): ModuleManifest[] {
  if (!Array.isArray(value)) fail();
  return value.map(parseManifest);
}
