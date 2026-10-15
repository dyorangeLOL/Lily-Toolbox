import { readFileSync } from "node:fs";
import { validateEvents } from "./fixture-server.mjs";
const [file, count, mode] = process.argv.slice(2);
const events = JSON.parse(readFileSync(file, "utf8").replace(/^\uFEFF/, ""));
console.log(JSON.stringify(validateEvents(events, Number(count), { fresh: mode === "fresh", persistent: mode === "persistent" })));
