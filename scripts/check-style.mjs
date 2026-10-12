// Keeps the stylesheets on the Colony design system:
// - no literal colour: every colour is a --colony-* palette variable
// - no raw font size: every size is a --fs-NN step or scales with --font-scale
// - only the weights the font ships (400, 500, 700; 900 is Font Awesome
//   Solid) and no italics, so the browser never synthesizes a face
//
//   npm run check:style

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));

function* files(dir) {
  for (const entry of readdirSync(join(root, dir), { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* files(path);
    else if (/\.(css|svelte)$/.test(entry.name)) yield path;
  }
}

const rules = [
  [/#[0-9a-f]{3,8}\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(/gi, "literal colour, use a --colony-* variable"],
  [/font-size:\s*(?![^;]*var\(--(?:fs-\d+|font-scale)\))[^;]*/g, "raw font-size, use var(--fs-NN)"],
  [/--fs-[\w-]+:\s*(?![^;]*var\(--font-scale\))[^;]*/g, "type step that ignores --font-scale"],
  [/font-weight:\s*(?=\S)(?!var\(--fw-(?:regular|medium|bold)\)|400\b|500\b|700\b|900\b|100 900\b)[^;]*/g, "weight the font does not ship"],
  [/font-style:\s*(?:italic|oblique)/g, "italic, which the font does not ship"],
];

const errors = [];
for (const file of files("src")) {
  const text = readFileSync(join(root, file), "utf8");
  for (const [pattern, why] of rules) {
    for (const match of text.matchAll(pattern)) {
      const line = text.slice(0, match.index).split("\n").length;
      errors.push(`${file}:${line}: ${match[0].trim()}: ${why}`);
    }
  }
}

if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log("stylesheets use only palette colours, scaled sizes and shipped weights");
