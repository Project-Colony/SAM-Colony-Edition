// English is canonical; French is a full translation. SAM's own strings live in
// en.json / fr.json. The Preferences skeleton strings every Colony program
// shares come from colony-ui, vendored unchanged in src/lib/vendor/, and are
// never copied into SAM's files. scripts/check-i18n.mjs keeps both locales in
// step.
import en from "./en.json";
import fr from "./fr.json";
import sharedEn from "../lib/vendor/labels.en.json";
import sharedFr from "../lib/vendor/labels.fr.json";

export type Locale = "en" | "fr";

const tables: Record<Locale, Record<string, string>> = {
  en: { ...sharedEn, ...en },
  fr: { ...sharedFr, ...fr },
};

const current = $state({ locale: localeFromTag(navigator.language) });

/** "fr", "fr-FR" and "fr_CA" give French; anything else English. */
export function localeFromTag(tag: string): Locale {
  return tag.toLowerCase().startsWith("fr") ? "fr" : "en";
}

export function locale(): Locale {
  return current.locale;
}

export function setLocale(next: Locale) {
  current.locale = next;
  document.documentElement.lang = next;
}

/** The string for `key` in the current locale, with `{name}` placeholders filled. */
export function t(key: string, params: Record<string, string | number> = {}): string {
  const text = tables[current.locale][key] ?? tables.en[key] ?? key;
  return text.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

/** A command error in words: the translated code, then Steam's or the OS's own detail. */
export function describeError(err: unknown): string {
  if (typeof err === "object" && err !== null && "code" in err) {
    const { code, detail } = err as { code: string; detail?: string };
    return detail ? `${t(code)} (${detail})` : t(code);
  }
  return String(err);
}
