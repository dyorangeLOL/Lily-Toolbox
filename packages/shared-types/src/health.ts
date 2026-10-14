export interface HealthStatus {
  api_version: 1;
  version: string;
  runtime: "tauri-rust";
}

export function parseHealth(value: unknown): HealthStatus {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Invalid native health response");
  }
  const object = value as Record<string, unknown>;
  if (
    Object.keys(object).sort().join(",") !== "api_version,runtime,version" ||
    object.api_version !== 1 ||
    object.runtime !== "tauri-rust" ||
    typeof object.version !== "string" ||
    !/^\d+\.\d+\.\d+$/.test(object.version)
  ) {
    throw new Error("Unsupported native health contract");
  }
  return object as unknown as HealthStatus;
}
