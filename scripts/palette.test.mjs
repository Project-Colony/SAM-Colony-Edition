// node --test scripts/
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { contrastOn, contrastRatio, resolveTheme, withHighContrast } from "../src/lib/palette.js";

const themes = JSON.parse(readFileSync(new URL("../src/lib/vendor/themes.json", import.meta.url), "utf8"));
const variant = (family, key) =>
  themes.families.find((f) => f.key === family).variants.find((v) => v.key === key);

test("a known theme resolves to itself", () => {
  assert.equal(resolveTheme(themes.families, "nord", "light"), variant("nord", "light"));
});

test("an unknown family or variant falls back to Gruvbox dark", () => {
  const gruvboxDark = variant("gruvbox", "dark");
  assert.equal(resolveTheme(themes.families, "nope", "dark"), gruvboxDark);
  assert.equal(resolveTheme(themes.families, "nord", "nope"), gruvboxDark);
  assert.equal(resolveTheme(themes.families, "", ""), gruvboxDark);
});

test("high contrast pushes text away from the background and keeps surfaces", () => {
  for (const [family, key] of [["gruvbox", "dark"], ["gruvbox", "light"], ["parchment", "light"]]) {
    const base = variant(family, key).palette;
    const boosted = withHighContrast(base);
    assert.equal(boosted.bg_primary, base.bg_primary);
    for (const field of ["text_primary", "text_muted", "divider"]) {
      assert.ok(
        contrastRatio(boosted[field], base.bg_primary) >= contrastRatio(base[field], base.bg_primary),
        `${family}-${key} ${field}`,
      );
    }
  }
  // colony-ui boosts each channel by 0.12 on a dark theme: 0xeb + 31 clamps to 0xff.
  assert.equal(withHighContrast(variant("gruvbox", "dark").palette).text_primary, "#fffad1");
});

test("contrastOn picks the legible end and clears 4.0:1 on every theme and accent", () => {
  assert.equal(contrastOn("#ffffff"), "#14141a");
  assert.equal(contrastOn("#000000"), "#f7faff");
  // The green accent is where a luminance threshold used to pick white.
  assert.equal(contrastOn("#55b87a"), "#14141a");

  const fills = [
    ...themes.families.flatMap((f) => f.variants.map((v) => v.palette.accent_blue)),
    ...themes.accents.map((a) => a.color),
  ];
  for (const fill of fills) {
    assert.ok(contrastRatio(contrastOn(fill), fill) >= 4.0, fill);
  }
});
