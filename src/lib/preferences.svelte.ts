// The user's preferences and how they take effect. Rust owns the file
// (src-tauri/src/preferences.rs); this module holds the live copy, puts it
// into effect on <html> and saves every change at once: there is no Save
// button.
import { invoke } from "@tauri-apps/api/core";
import themes from "./vendor/themes.json";
import { contrastOn, resolveTheme, withHighContrast } from "./palette.js";
import { localeFromTag, setLocale } from "../i18n/index.svelte";

export type FontSize = "small" | "default" | "large";
export type TextSize = FontSize | "xlarge";

/** Field names and values match colony-ui's StandardPreferences, plus `language`. */
export interface Preferences {
  selected_theme: string;
  selected_variant: string;
  selected_accent?: string | null;
  auto_accent: boolean;
  font_size: FontSize;
  animations: boolean;
  high_contrast: boolean;
  dyslexia_font: boolean;
  reduce_motion: boolean;
  text_size_a11y: TextSize;
  /** Absent follows the system language. */
  language?: "en" | "fr" | null;
}

export const FONT_SIZES: Record<FontSize, number> = { small: 0.85, default: 1, large: 1.2 };
export const TEXT_SIZES: Record<TextSize, number> = { ...FONT_SIZES, xlarge: 1.4 };

export const prefs: Preferences = $state({
  selected_theme: "gruvbox",
  selected_variant: "dark",
  auto_accent: false,
  font_size: "default",
  animations: true,
  high_contrast: false,
  dyslexia_font: false,
  reduce_motion: false,
  text_size_a11y: "default",
});

/** The theme in effect: the stored one, or Gruvbox dark when it is unknown. */
export function activeVariant() {
  return resolveTheme(themes.families, prefs.selected_theme, prefs.selected_variant);
}

/** The accent key the picker marks: none while auto accent is on. */
export function pickedAccent(): string | null {
  return prefs.auto_accent ? null : (prefs.selected_accent ?? null);
}

function apply() {
  const root = document.documentElement;
  const variant = activeVariant();
  const palette: Record<string, string> = prefs.high_contrast
    ? withHighContrast(variant.palette)
    : variant.palette;
  for (const field of themes.fields) root.style.setProperty(field.cssVar, palette[field.name]);

  const accent =
    themes.accents.find((a) => a.key === pickedAccent())?.color ?? palette.accent_blue;
  root.style.setProperty("--colony-accent", accent);
  root.style.setProperty("--colony-on-accent", contrastOn(accent));
  root.style.colorScheme = variant.mode;

  root.style.setProperty("--font-size-pref", String(FONT_SIZES[prefs.font_size]));
  root.style.setProperty("--text-size-pref", String(TEXT_SIZES[prefs.text_size_a11y]));
  root.toggleAttribute("data-dyslexia", prefs.dyslexia_font);
  // Effects degrade to nothing under reduced motion; the stylesheet also
  // honours the system's prefers-reduced-motion.
  root.toggleAttribute("data-reduced-motion", prefs.reduce_motion || !prefs.animations);

  setLocale(prefs.language ?? localeFromTag(navigator.language));
}

/** Load the saved preferences and put them into effect, before the first paint. */
export async function loadPreferences() {
  try {
    Object.assign(prefs, await invoke<Preferences>("cmd_load_preferences"));
  } catch (err) {
    console.error("loading preferences failed, using the defaults", err);
  }
  apply();
}

/** Apply a change now and save it. Rejects with the save error, if any. */
export function updatePreferences(change: Partial<Preferences>): Promise<void> {
  Object.assign(prefs, change);
  apply();
  return invoke<void>("cmd_save_preferences", { preferences: $state.snapshot(prefs) });
}
