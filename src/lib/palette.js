// @ts-check
// Palette logic mirrored from colony-ui's theme module (Project-Colony-Resources,
// crates/colony-ui/src/theme.rs): theme resolution with the Gruvbox dark
// fallback, the high-contrast derivation and contrast_on. Plain JavaScript so
// scripts/palette.test.mjs can run it with node:test and no build step.

/** The theme a fresh install starts on and an unknown theme falls back to. */
export const FALLBACK_THEME = { family: "gruvbox", variant: "dark" };

/**
 * The variant for a stored (family, variant) pair, or Gruvbox dark when this
 * build does not know the pair. The stored keys are left as written, so a
 * later build that knows the theme gets the user's choice back.
 *
 * @template {{ key: string }} V
 * @param {{ key: string, variants: V[] }[]} families
 * @param {string} family
 * @param {string} variant
 * @returns {V}
 */
export function resolveTheme(families, family, variant) {
  /** @param {string} f @param {string} v */
  const find = (f, v) => families.find((x) => x.key === f)?.variants.find((x) => x.key === v);
  const found = find(family, variant) ?? find(FALLBACK_THEME.family, FALLBACK_THEME.variant);
  if (!found) throw new Error("the theme catalog has no Gruvbox dark");
  return found;
}

/** @typedef {[number, number, number]} Rgb channels in 0..1 */

/** @param {string} hex `#rrggbb` @returns {Rgb} */
function parse(hex) {
  const n = parseInt(hex.slice(1, 7), 16);
  return [(n >> 16) / 255, ((n >> 8) & 0xff) / 255, (n & 0xff) / 255];
}

/** @param {Rgb} rgb @returns {string} */
function format(rgb) {
  return "#" + rgb.map((c) => Math.round(c * 255).toString(16).padStart(2, "0")).join("");
}

/** YIQ luminance, the "is this surface light?" test colony-ui uses. @param {Rgb} c */
function isLight([r, g, b]) {
  return 0.299 * r + 0.587 * g + 0.114 * b > 0.5;
}

/** @param {string} hex @param {number} amount */
function shift(hex, amount) {
  const [r, g, b] = parse(hex);
  /** @param {number} c */
  const clamp = (c) => Math.min(1, Math.max(0, c + amount));
  return format([clamp(r), clamp(g), clamp(b)]);
}

/**
 * colony-ui's `with_high_contrast`: text and borders pushed away from the
 * background, derived from the active palette so no theme ships a twin.
 *
 * @param {Record<string, string>} palette
 * @returns {Record<string, string>}
 */
export function withHighContrast(palette) {
  const light = isLight(parse(palette.bg_primary));
  /** @type {[string, number, number][]} field, darken on light, boost on dark */
  const steps = [
    ["text_primary", 0.15, 0.12],
    ["text_secondary", 0.12, 0.1],
    ["text_muted", 0.1, 0.1],
    ["text_dim", 0.1, 0.08],
    ["text_dimmer", 0.08, 0.08],
    ["border_subtle", 0.15, 0.12],
    ["divider", 0.15, 0.12],
  ];
  const out = { ...palette };
  for (const [field, darken, boost] of steps) {
    out[field] = shift(palette[field], light ? -darken : boost);
  }
  return out;
}

/** @param {Rgb} c WCAG relative luminance */
function luminance(c) {
  const [r, g, b] = c.map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio, 1 to 21. @param {string} a @param {string} b */
export function contrastRatio(a, b) {
  const [la, lb] = [luminance(parse(a)), luminance(parse(b))];
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

// colony-ui's INK (0.08, 0.08, 0.10) and PAPER (0.97, 0.98, 1.0) in hex.
const INK = "#14141a";
const PAPER = "#f7faff";

/**
 * A legible near-black or near-white for text drawn on `bg`, whichever has the
 * higher WCAG contrast (colony-ui `contrast_on`).
 *
 * @param {string} bg `#rrggbb`
 */
export function contrastOn(bg) {
  return contrastRatio(INK, bg) >= contrastRatio(PAPER, bg) ? INK : PAPER;
}
