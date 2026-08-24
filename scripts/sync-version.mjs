import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

export function readWorkspaceVersion() {
  const toml = readFileSync(join(root, "Cargo.toml"), "utf8");
  const m = toml.match(/\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/);
  if (!m) {
    throw new Error("未在 Cargo.toml 中找到 [workspace.package] version");
  }
  return m[1];
}

function syncJsonVersion(path, version) {
  const json = JSON.parse(readFileSync(path, "utf8"));
  if (json.version === version) {
    return;
  }
  json.version = version;
  writeFileSync(path, `${JSON.stringify(json, null, 2)}\n`);
}

function syncTauriConfVersion(path, version) {
  const raw = readFileSync(path, "utf8");
  const next = raw.replace(/"version"\s*:\s*"[^"]+"/, `"version": "${version}"`);
  if (next !== raw) {
    writeFileSync(path, next);
  }
}

const version = readWorkspaceVersion();
syncJsonVersion(join(root, "app-tauri/package.json"), version);
syncTauriConfVersion(join(root, "app-tauri/src-tauri/tauri.conf.json"), version);
console.log(`version synced: ${version}`);
