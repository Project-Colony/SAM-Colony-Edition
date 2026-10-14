// Fails when the English and French tables drift apart, when a string is
// empty, when SAM redefines a shared colony-ui key, or when the UI or Rust uses
// a key that does not exist.
//
//   npm run check:i18n

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const json = (path) => JSON.parse(readFileSync(join(root, path), "utf8"));
const errors = [];

const own = { en: json("src/i18n/en.json"), fr: json("src/i18n/fr.json") };
const shared = { en: json("src/lib/vendor/labels.en.json"), fr: json("src/lib/vendor/labels.fr.json") };

for (const tables of [own, shared]) {
  for (const [a, b] of [["en", "fr"], ["fr", "en"]]) {
    for (const key of Object.keys(tables[a])) {
      if (!(key in tables[b])) errors.push(`"${key}" is in ${a} but not in ${b}`);
      if (!tables[a][key].trim()) errors.push(`"${key}" is empty in ${a}`);
    }
  }
}
for (const key of Object.keys(own.en)) {
  if (key in shared.en) errors.push(`"${key}" redefines a shared colony-ui label; use the shared one`);
}

function* files(dir, ext) {
  for (const entry of readdirSync(join(root, dir), { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* files(path, ext);
    else if (ext.some((e) => entry.name.endsWith(e))) yield path;
  }
}

const known = (key) => key in own.en || key in shared.en;
const uses = [
  // t("key") and t("key", {...}) in the frontend.
  ...[...files("src", [".svelte", ".ts", ".js"])].map((f) => [f, /\bt\(\s*"([a-z0-9_]+)"/g]),
  // Error and status codes the Rust commands hand the UI.
  ...[...files("src-tauri/src", [".rs"])].map((f) => [f, /"((?:error|status)_[a-z0-9_]+)"/g]),
];
for (const [file, pattern] of uses) {
  for (const [, key] of readFileSync(join(root, file), "utf8").matchAll(pattern)) {
    if (!known(key)) errors.push(`${file} uses "${key}", which has no string`);
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`${Object.keys(own.en).length} SAM strings and ${Object.keys(shared.en).length} shared labels, en and fr in step`);
