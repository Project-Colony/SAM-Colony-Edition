<script lang="ts">
  // The Colony Preferences page (design/settings-page.md), built to the same
  // measurements as colony-ui's PreferencesView. It replaces the content area;
  // the sidebar stays. Every change applies and saves at once.
  import { onMount, type Snippet } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { getVersion } from "@tauri-apps/api/app";
  import { invoke } from "@tauri-apps/api/core";
  import themes from "../lib/vendor/themes.json";
  import { contrastOn } from "../lib/palette.js";
  import { CHECK, CHEVRON_DOWN, CHEVRON_RIGHT, CLOSE } from "../lib/icons";
  import { describeError, locale, t } from "../i18n/index.svelte";
  import {
    FONT_SIZES,
    TEXT_SIZES,
    activeVariant,
    pickedAccent,
    prefs,
    updatePreferences,
    type FontSize,
    type Preferences,
    type TextSize,
  } from "../lib/preferences.svelte";
  import type { Message } from "../lib/types";

  let { onClose, onStatus }: { onClose: () => void; onStatus: (msg: Message) => void } =
    $props();

  // General, Appearance, Accessibility in that order, then About last.
  const categories = [
    () => t("settings_cat_general"),
    () => t("settings_cat_appearance"),
    () => t("settings_cat_accessibility"),
    () => t("settings_cat_about"),
  ];
  let category = $state(0);
  // Open sections, remembered while the page is open. All start collapsed.
  const expanded = new SvelteSet<string>();
  let themeToast = $state(false);
  let version = $state("");
  let storage: { config: string | null; cache: string | null } = $state({ config: null, cache: null });

  onMount(async () => {
    version = await getVersion().catch(() => "");
    storage = await invoke<typeof storage>("cmd_storage_paths").catch(() => storage);
  });

  function toggleSection(key: string) {
    if (!expanded.delete(key)) expanded.add(key);
  }

  function change(patch: Partial<Preferences>) {
    updatePreferences(patch).catch((err) => onStatus(() => describeError(err)));
  }

  function pickTheme(family: string, variant: string) {
    change({ selected_theme: family, selected_variant: variant });
    // Switching to a neighbouring variant can be hard to notice.
    themeToast = true;
  }

  const label = (l: { en: string; fr: string }) => l[locale()];
  const glyph = (codepoint: string) => String.fromCodePoint(parseInt(codepoint, 16));

  const fontSizes = Object.keys(FONT_SIZES) as FontSize[];
  const textSizes = Object.keys(TEXT_SIZES) as TextSize[];
</script>

{#snippet section(key: string, title: string, description: string, body: Snippet)}
  <div class="section">
    <button class="section__header" aria-expanded={expanded.has(key)} onclick={() => toggleSection(key)}>
      <span class="section__title">{title}</span>
      <span class="glyph section__chevron" aria-hidden="true">
        {expanded.has(key) ? CHEVRON_DOWN : CHEVRON_RIGHT}
      </span>
    </button>
    {#if expanded.has(key)}
      <div class="section__body">
        <p class="section__desc">{description}</p>
        {@render body()}
      </div>
    {/if}
  </div>
{/snippet}

{#snippet toggle(title: string, description: string, on: boolean, flip: () => void, disabled = false)}
  <button class="row row--button" role="switch" aria-checked={on} {disabled} onclick={flip}>
    <span class="row__text">
      <span class="row__title">{title}</span>
      <span class="row__desc">{description}</span>
    </span>
    <span class="switch" class:switch--on={on} aria-hidden="true"><span class="switch__knob"></span></span>
  </button>
{/snippet}

{#snippet choice(title: string, description: string, value: string, options: [string, string][], pick: (value: string) => void)}
  <label class="row">
    <span class="row__text">
      <span class="row__title">{title}</span>
      <span class="row__desc">{description}</span>
    </span>
    <select class="select" {value} onchange={(e) => pick(e.currentTarget.value)}>
      {#each options as [option, text] (option)}
        <option value={option}>{text}</option>
      {/each}
    </select>
  </label>
{/snippet}

{#snippet heading(title: string, description: string)}
  <header class="heading">
    <h2 class="heading__title">{title}</h2>
    <p class="heading__desc">{description}</p>
  </header>
{/snippet}

{#snippet language()}
  {@render choice(
    t("preferences_language"),
    t("preferences_language_desc"),
    prefs.language ?? "",
    [
      ["", t("preferences_language_system")],
      ["en", t("preferences_language_en")],
      ["fr", t("preferences_language_fr")],
    ],
    (value) => change({ language: value === "en" || value === "fr" ? value : null }),
  )}
{/snippet}

{#snippet themePicker()}
  {@const active = activeVariant()}
  <div class="families">
    {#each themes.families as family (family.key)}
      <div class="family">
        <div class="family__label" class:family__label--active={family.variants.some((v) => v === active)}>
          {#if family.icon}<span class="glyph" aria-hidden="true">{glyph(family.icon)}</span>{/if}
          <span>{label(family.label)}</span>
        </div>
        <div class="variants">
          {#each family.variants as variant (variant.key)}
            <button
              class="variant"
              class:variant--active={variant === active}
              aria-pressed={variant === active}
              onclick={() => pickTheme(family.key, variant.key)}
            >
              <span class="variant__swatch" style:background={variant.swatch.bg}>
                {#if variant === active}
                  <span class="glyph variant__check" style:color={contrastOn(variant.swatch.bg)} aria-hidden="true">{CHECK}</span>
                {/if}
                <span class="variant__bar" style:background={variant.swatch.accent}></span>
              </span>
              <span class="variant__name">{label(variant.label)}</span>
            </button>
          {/each}
        </div>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet colors()}
  <div class="accents">
    {#each themes.accents as accent (accent.key)}
      {@const on = pickedAccent() === accent.key}
      <button
        class="accent"
        class:accent--active={on}
        style:background={accent.color}
        style:color={contrastOn(accent.color)}
        title={label(accent.label)}
        aria-label={label(accent.label)}
        aria-pressed={on}
        onclick={() => change({ selected_accent: accent.key, auto_accent: false })}
      >
        {#if on}<span class="glyph" aria-hidden="true">{CHECK}</span>{/if}
      </button>
    {/each}
  </div>
  {@render toggle(t("settings_auto_accent"), t("settings_auto_accent_desc"), prefs.auto_accent, () =>
    change({ auto_accent: !prefs.auto_accent }),
  )}
{/snippet}

{#snippet typography()}
  <div class="row">
    <span class="row__text">
      <span class="row__title">{t("settings_font_family")}</span>
      <span class="row__desc">{t("settings_font_family_desc")}</span>
    </span>
    <span class="row__value">{prefs.dyslexia_font ? "OpenDyslexic" : "JetBrainsMono Nerd Font"}</span>
  </div>
  {@render choice(
    t("settings_font_size"),
    t("settings_font_size_desc"),
    prefs.font_size,
    fontSizes.map((size) => [size, t(`settings_font_size_${size}`)]),
    (value) => change({ font_size: value as FontSize }),
  )}
{/snippet}

{#snippet effects()}
  {@render toggle(
    t("settings_animations"),
    t("settings_animations_desc"),
    prefs.animations && !prefs.reduce_motion,
    () => change({ animations: !prefs.animations }),
    prefs.reduce_motion,
  )}
{/snippet}

{#snippet preview()}
  {@const variant = activeVariant()}
  {@const accent = themes.accents.find((a) => a.key === pickedAccent())}
  <div class="preview">
    <div class="preview__title">{t("settings_preview_card")}</div>
    <div class="preview__body">{t("settings_preview_body")}</div>
    <div class="preview__meta">
      {themes.families.find((f) => f.variants.some((v) => v === variant))?.label[locale()]}
      · {label(variant.label)} · {accent ? label(accent.label) : t("settings_default_accent")}
    </div>
    <span class="btn btn--primary preview__button">{t("settings_preview_button")}</span>
  </div>
{/snippet}

{#snippet vision()}
  {@render toggle(t("settings_high_contrast"), t("settings_high_contrast_desc"), prefs.high_contrast, () =>
    change({ high_contrast: !prefs.high_contrast }),
  )}
  {@render toggle(t("settings_dyslexia_font"), t("settings_dyslexia_font_desc"), prefs.dyslexia_font, () =>
    change({ dyslexia_font: !prefs.dyslexia_font }),
  )}
{/snippet}

{#snippet motion()}
  {@render toggle(t("settings_reduce_motion"), t("settings_reduce_motion_desc"), prefs.reduce_motion, () =>
    change({ reduce_motion: !prefs.reduce_motion }),
  )}
{/snippet}

{#snippet reading()}
  {@render choice(
    t("settings_text_size_a11y"),
    t("settings_text_size_a11y_desc"),
    prefs.text_size_a11y,
    textSizes.map((size) => [size, t(`settings_font_size_${size}`)]),
    (value) => change({ text_size_a11y: value as TextSize }),
  )}
{/snippet}

<div class="prefs">
  <header class="prefs__header">
    <h1 class="prefs__title">{t("settings_title")}</h1>
    <button class="prefs__close" onclick={onClose}>{t("settings_close")}</button>
  </header>

  <div class="prefs__body">
    <nav class="categories">
      {#each categories as name, index (index)}
        <button
          class="categories__item"
          class:categories__item--selected={index === category}
          aria-current={index === category ? "page" : undefined}
          onclick={() => (category = index)}
        >
          {name()}
        </button>
      {/each}
    </nav>

    <div class="prefs__content">
      {#if category === 0}
        {@render heading(t("settings_cat_general"), t("settings_general_desc"))}
        {@render section("language", t("preferences_section_language"), t("preferences_language_desc"), language)}
      {:else if category === 1}
        {@render heading(t("settings_cat_appearance"), t("settings_appearance_desc"))}
        {@render section("theme", t("settings_section_theme"), t("settings_theme_desc"), themePicker)}
        {@render section("colors", t("settings_section_colors"), t("settings_colors_desc"), colors)}
        {@render section("typography", t("settings_section_typography"), t("settings_typography_desc"), typography)}
        {@render section("effects", t("settings_section_effects"), t("settings_effects_desc"), effects)}
        {@render section("preview", t("settings_section_preview"), t("settings_preview_desc"), preview)}
      {:else if category === 2}
        {@render heading(t("settings_cat_accessibility"), t("settings_accessibility_desc"))}
        {@render section("vision", t("settings_section_vision"), t("settings_vision_desc"), vision)}
        {@render section("motion", t("settings_section_motion"), t("settings_motion_desc"), motion)}
        {@render section("reading", t("settings_section_reading"), t("settings_reading_desc"), reading)}
      {:else}
        {@render heading(t("settings_cat_about"), t("preferences_about_desc"))}
        <dl class="about">
          <dt>{t("preferences_about_version")}</dt>
          <dd>{version}</dd>
          <dt>{t("preferences_about_license")}</dt>
          <dd>{t("preferences_about_license_value")}</dd>
          <dt>{t("preferences_about_source")}</dt>
          <dd class="about__path">https://github.com/Project-Colony/SAM-Colony-Edition</dd>
          {#if storage.config}
            <dt>{t("preferences_about_preferences_dir")}</dt>
            <dd class="about__path">{storage.config}</dd>
          {/if}
          {#if storage.cache}
            <dt>{t("preferences_about_cache_dir")}</dt>
            <dd class="about__path">{storage.cache}</dd>
          {/if}
        </dl>
      {/if}
    </div>
  </div>

  {#if themeToast}
    <button class="toast" onclick={() => (themeToast = false)} title={t("preferences_toast_dismiss")}>
      <span>{t("theme_applied")}</span>
      <span class="glyph toast__close" aria-hidden="true">{CLOSE}</span>
    </button>
  {/if}
</div>

<style>
  .prefs {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 16px;
    flex: 1;
    min-height: 0;
  }

  .prefs__header { display: flex; align-items: center; gap: 12px; }
  .prefs__title {
    margin: 0;
    flex: 1;
    font-size: var(--fs-22);
    font-weight: var(--fw-bold);
    color: var(--colony-text-primary);
  }
  .prefs__close {
    font-size: var(--fs-13);
    color: var(--colony-text-muted);
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    cursor: pointer;
  }
  .prefs__close:hover { background: var(--colony-bg-card-hover); }

  .prefs__body {
    display: grid;
    grid-template-columns: calc(180px * var(--font-scale)) minmax(0, 1fr);
    gap: 16px;
    flex: 1;
    min-height: 0;
  }

  .categories { display: flex; flex-direction: column; gap: 2px; }
  .categories__item {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 8px 14px;
    font-size: var(--fs-13);
    color: var(--colony-text-muted);
    cursor: pointer;
    overflow-wrap: anywhere;
  }
  .categories__item:hover { background: var(--colony-bg-card-hover); }
  .categories__item--selected,
  .categories__item--selected:hover {
    background: var(--colony-accent);
    color: var(--colony-on-accent);
  }

  .prefs__content {
    overflow: auto;
    padding: 0 24px 24px 0;
  }

  .heading { padding-bottom: 14px; }
  .heading__title {
    margin: 0 0 4px;
    font-size: var(--fs-18);
    font-weight: var(--fw-bold);
    color: var(--colony-text-primary);
  }
  .heading__desc { margin: 0; font-size: var(--fs-12); color: var(--colony-text-muted); }

  /* A flat header row: no box, the whole row toggles. */
  .section__header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 4px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: var(--colony-text-primary);
    cursor: pointer;
    text-align: left;
  }
  .section__header:hover { background: var(--colony-bg-card-hover); }
  .section__title { flex: 1; font-size: var(--fs-15); font-weight: var(--fw-bold); }
  .section__chevron { font-size: var(--fs-10); color: var(--colony-text-dimmer); }
  .section__body {
    border-top: 1px solid var(--colony-divider);
    padding: 12px 4px 4px;
  }
  .section__desc { margin: 0 0 12px; font-size: var(--fs-12); color: var(--colony-text-muted); }

  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 4px;
    border-radius: 6px;
    text-align: left;
    color: var(--colony-text-primary);
  }
  .row--button {
    background: transparent;
    border: none;
    cursor: pointer;
  }
  .row--button:hover { background: var(--colony-bg-card-hover); }
  .row--button:disabled { cursor: default; opacity: 0.6; }
  .row--button:disabled:hover { background: transparent; }
  .row__text { flex: 1; display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .row__title { font-size: var(--fs-13); }
  .row__desc { font-size: var(--fs-11); color: var(--colony-text-dimmer); }
  .row__value { font-size: var(--fs-12); color: var(--colony-text-secondary); }

  .switch {
    flex-shrink: 0;
    width: 34px;
    height: 18px;
    padding: 2px;
    border-radius: 9px;
    background: var(--colony-bg-card-hover);
    transition: background 120ms ease;
  }
  .switch--on { background: var(--colony-accent); }
  .switch__knob {
    display: block;
    width: 14px;
    height: 14px;
    border-radius: 7px;
    background: var(--colony-text-primary);
    transition: transform 120ms ease;
  }
  .switch--on .switch__knob { transform: translateX(16px); }

  .select {
    font-size: var(--fs-12);
    color: var(--colony-text-secondary);
    background: var(--colony-bg-card);
    border: 1px solid var(--colony-border-subtle);
    border-radius: 6px;
    padding: 4px 10px;
    max-width: 50%;
  }
  .select:hover { background: var(--colony-bg-card-hover); }

  .families { display: flex; flex-direction: column; gap: 14px; }
  .family__label {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
    font-size: var(--fs-13);
    font-weight: var(--fw-medium);
    color: var(--colony-text-secondary);
  }
  .family__label--active { color: var(--colony-text-primary); }
  .variants {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(calc(110px * var(--font-scale)), 1fr));
    gap: 8px;
  }
  .variant {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px;
    background: var(--colony-bg-card);
    border: 1px solid var(--colony-border-subtle);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
  }
  .variant:hover { border-color: var(--colony-text-dimmer); }
  .variant--active,
  .variant--active:hover {
    border: 2px solid var(--colony-accent);
    padding: 3px;
  }
  /* The variant's own background, crossed by a thin bar in its accent. */
  .variant__swatch {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    height: 28px;
    padding: 4px 6px;
    border-radius: 6px;
  }
  .variant__bar { height: 4px; border-radius: 2px; }
  .variant__check { position: absolute; top: 3px; right: 6px; font-size: var(--fs-10); }
  .variant__name { padding-left: 2px; font-size: var(--fs-10); color: var(--colony-text-muted); }
  .variant--active .variant__name { color: var(--colony-text-primary); }

  .accents { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 12px; }
  .accent {
    width: 28px;
    height: 28px;
    border-radius: 14px;
    border: 2px solid transparent;
    display: grid;
    place-items: center;
    font-size: var(--fs-10);
    cursor: pointer;
  }
  .accent:hover { border-color: var(--colony-text-dimmer); }
  .accent--active,
  .accent--active:hover { border-color: var(--colony-text-primary); }

  .preview {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 16px;
    background: var(--colony-bg-card);
    border: 1px solid var(--colony-border-subtle);
    border-radius: 8px;
  }
  .preview__title { font-size: var(--fs-14); font-weight: var(--fw-bold); }
  .preview__body { font-size: var(--fs-12); color: var(--colony-text-secondary); }
  .preview__meta { font-size: var(--fs-11); color: var(--colony-text-muted); }
  .preview__button { cursor: default; font-size: var(--fs-12); }

  .about {
    display: grid;
    grid-template-columns: max-content minmax(0, 1fr);
    gap: 8px 16px;
    margin: 0;
    font-size: var(--fs-13);
  }
  .about dt { color: var(--colony-text-muted); }
  .about dd { margin: 0; color: var(--colony-text-primary); }
  .about__path { overflow-wrap: anywhere; user-select: text; }

  .toast {
    position: absolute;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    background: var(--colony-success-bg);
    border: none;
    border-radius: 8px;
    font-size: var(--fs-13);
    color: var(--colony-success);
    cursor: pointer;
  }
  .toast__close { font-size: var(--fs-12); color: var(--colony-text-dimmer); }
</style>
