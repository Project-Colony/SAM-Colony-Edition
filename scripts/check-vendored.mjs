// Verifies that every file vendored from Project-Colony-Resources is
// byte-identical to the upstream file at the commit recorded in
// scripts/vendored.json.
//
//   node scripts/check-vendored.mjs                  check (CI)
//   node scripts/check-vendored.mjs --update <sha>   re-vendor everything from <sha>

import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const manifestPath = `${root}scripts/vendored.json`;
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));

const update = process.argv[2] === "--update";
if (update) {
  if (!/^[0-9a-f]{40}$/.test(process.argv[3] ?? "")) {
    console.error("usage: --update <full 40-character commit sha>");
    process.exit(2);
  }
  manifest.commit = process.argv[3];
}

async function upstream(path) {
  const url = `https://raw.githubusercontent.com/${manifest.repository}/${manifest.commit}/${path}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  return Buffer.from(await res.arrayBuffer());
}

let failed = 0;
for (const [local, remote] of Object.entries(manifest.files)) {
  const expected = await upstream(remote);
  if (update) {
    await writeFile(`${root}${local}`, expected);
    continue;
  }
  const actual = await readFile(`${root}${local}`).catch(() => null);
  if (actual === null || !actual.equals(expected)) {
    console.error(`${local} differs from ${remote} at ${manifest.commit}`);
    failed++;
  }
}

if (update) {
  await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + "\n");
  console.log(`re-vendored ${Object.keys(manifest.files).length} files from ${manifest.commit}`);
} else if (failed > 0) {
  console.error(`${failed} vendored file(s) out of date; run with --update <sha> to refresh them`);
  process.exit(1);
} else {
  console.log(`${Object.keys(manifest.files).length} vendored files match ${manifest.commit}`);
}
