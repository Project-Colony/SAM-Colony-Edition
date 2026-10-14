<script lang="ts">
  import { describeError, t } from "../i18n/index.svelte";
  import { cmd } from "../lib/tauri";
  import type { Achievement, Message } from "../lib/types";

  let {
    achievements,
    icons,
    onStatus,
    onReload,
  }: {
    achievements: Achievement[];
    icons: Record<string, string>;
    onStatus: (msg: Message | Message[]) => void;
    onReload: (refresh: boolean) => void;
  } = $props();

  let items: Achievement[] = $state([]);
  let filter = $state("");
  let debounceTimer: number | null = null;

  // Reset local working copy whenever upstream achievements refresh.
  // $state.snapshot() returns plain JS objects (structuredClone on a Svelte
  // proxy can return [] silently).
  $effect(() => {
    items = $state.snapshot(achievements).map((a) => ({ ...a }));
  });

  // Background preloader: when icons arrive, eagerly request every URL so
  // they sit in the WebKit HTTP cache before the user scrolls them into
  // view. Without this, lazy <img> loads fire a CDN burst on fast scroll.
  // Using new Image().src instead of fetch keeps the request inside the
  // image-loader pipeline (browser dedupes when <img src> later asks for
  // the same URL).
  let prefetchedKey = $state("");
  $effect(() => {
    const urls = Object.values(icons);
    if (urls.length === 0) return;
    const key = urls.length + "@" + urls[0];
    if (key === prefetchedKey) return;
    prefetchedKey = key;
    for (const url of urls) {
      const img = new Image();
      img.decoding = "async";
      img.referrerPolicy = "no-referrer";
      img.src = url;
    }
  });

  function handleFilter(event: Event) {
    filter = (event.target as HTMLInputElement).value;
    if (debounceTimer !== null) clearTimeout(debounceTimer);
    debounceTimer = window.setTimeout(applyFilter, 400);
  }

  function applyFilter() {
    const q = filter.trim().toLowerCase();
    const source = $state.snapshot(achievements).map((a) => ({ ...a }));
    if (q.length === 0) {
      items = source;
      return;
    }
    items = source.filter(
      (a) =>
        a.name.toLowerCase().includes(q) || a.desc.toLowerCase().includes(q)
    );
  }

  function toggle(idx: number) {
    items[idx].status = !items[idx].status;
  }

  function selectAll(newStatus: boolean) {
    items = items.map((a) => ({ ...a, status: newStatus }));
  }

  function showLockedOnly(locked: boolean) {
    items = $state.snapshot(achievements)
      .filter((a) => a.status === locked)
      .map((a) => ({ ...a }));
  }

  async function apply() {
    const alerts: Message[] = [];
    let written = 0;
    for (const it of items) {
      const original = achievements.find((a) => a.api_name === it.api_name);
      if (original && it.status !== original.status) {
        try {
          await cmd.commitAchievement(it.api_name, it.status);
          written++;
          const { name, status } = it;
          alerts.push(() => t(status ? "log_achievement_unlocked" : "log_achievement_locked", { name }));
        } catch (err) {
          const name = it.name;
          alerts.push(() => t("log_write_failed", { name, error: describeError(err) }));
        }
      }
    }
    // Upload only when Steam accepted a write: with nothing pending there is
    // nothing for the server to confirm.
    if (written > 0) {
      try {
        await cmd.storeStats();
      } catch (err) {
        alerts.unshift(() => t("log_upload_failed", { error: describeError(err) }));
      }
    }
    onStatus(alerts.length > 0 ? alerts : () => t("log_no_changes"));
    // Reload in every case: a rejected upload reverts Steam's local values.
    onReload(false);
  }

  let unlockedCount = $derived(items.filter((a) => a.status).length);

  // Font Awesome Solid, for an achievement whose icon Steam did not provide.
  const TROPHY = "\u{f091}";
  const LOCK = "\u{f023}";
</script>

<div class="ach">
  <header class="ach__toolbar panel">
    <input
      class="input ach__filter"
      type="text"
      placeholder={t("achievements_filter_placeholder")}
      oninput={handleFilter}
    />
    <div class="ach__count">
      <span class="count__num">{unlockedCount}</span>
      <span class="count__sep dim">/</span>
      <span class="count__total dim">{items.length}</span>
    </div>
    <div class="ach__buttons">
      <button class="btn" onclick={() => onReload(true)}>{t("action_refresh")}</button>
      <button class="btn btn--primary" onclick={apply}>{t("action_apply")}</button>
    </div>
  </header>

  <nav class="ach__subnav">
    <button class="btn" onclick={() => selectAll(true)}>{t("achievements_select_all")}</button>
    <button class="btn" onclick={() => selectAll(false)}>{t("achievements_deselect_all")}</button>
    <button class="btn" onclick={() => showLockedOnly(true)}>{t("achievements_unlocked_only")}</button>
    <button class="btn" onclick={() => showLockedOnly(false)}>{t("achievements_locked_only")}</button>
  </nav>

  {#if items.length === 0}
    <div class="panel ach__empty">
      <p class="muted">{t("achievements_none")}</p>
    </div>
  {:else}
    <ul class="ach__list">
      {#each items as item, idx (item.api_name)}
        <li>
          <label class="card" class:card--unlocked={item.status} class:card--locked={!item.status}>
            <input
              class="ach__checkbox"
              type="checkbox"
              checked={item.status}
              onchange={() => toggle(idx)}
            />
            <div class="ach__icon-wrap">
              {#if icons[item.api_name + (item.status ? "" : "-gray")]}
                <img
                  class="ach__icon"
                  alt=""
                  loading="lazy"
                  decoding="async"
                  src={icons[item.api_name + (item.status ? "" : "-gray")]}
                />
              {:else}
                <div class="ach__icon-fallback" class:ach__icon-fallback--locked={!item.status} aria-hidden="true">
                  {item.status ? TROPHY : LOCK}
                </div>
              {/if}
            </div>
            <div class="ach__body">
              <div class="ach__name">{item.name}</div>
              <div class="ach__desc muted">{item.desc}</div>
              <div class="ach__api dim">{item.api_name}</div>
            </div>
          </label>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .ach { display: flex; flex-direction: column; gap: 10px; min-height: 0; flex: 1; }

  .ach__toolbar {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    gap: 12px;
    align-items: center;
    padding: 10px 14px;
  }
  .ach__filter { max-width: calc(360px * var(--font-scale)); }
  .ach__count { display: flex; align-items: baseline; gap: 4px; }
  .count__num { font-size: var(--fs-18); font-weight: var(--fw-bold); color: var(--colony-text-primary); }
  .count__sep, .count__total { font-size: var(--fs-15); }
  .ach__buttons { display: flex; gap: 6px; flex-wrap: wrap; justify-content: flex-end; }

  .ach__subnav { display: flex; gap: 6px; flex-wrap: wrap; }

  .ach__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow: auto;
    min-height: 0;
  }
  .ach__list li { margin: 0; }
  .ach__list .card { cursor: pointer; align-items: center; }
  .ach__list .card:hover { background: var(--colony-bg-card-hover); }

  .ach__checkbox {
    appearance: none;
    width: calc(18px * var(--font-scale));
    height: calc(18px * var(--font-scale));
    border: 1px solid var(--colony-text-dimmer);
    border-radius: 4px;
    background: var(--colony-bg-input);
    cursor: pointer;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    margin: 0;
  }
  .ach__checkbox:checked {
    background: var(--colony-accent);
    border-color: var(--colony-accent);
  }
  .ach__checkbox:checked::after {
    content: "\f00c";
    font-family: var(--colony-glyph-font);
    color: var(--colony-on-accent);
    font-size: var(--fs-11);
    line-height: 1;
  }

  .ach__icon-wrap {
    width: 48px;
    height: 48px;
    border: 1px solid var(--colony-border-subtle);
    border-radius: var(--radius-input);
    overflow: hidden;
    background: var(--colony-bg-input);
    flex-shrink: 0;
    display: grid;
    place-items: center;
  }
  .ach__icon { width: 100%; height: 100%; object-fit: cover; display: block; }
  .ach__icon-fallback {
    color: var(--colony-accent);
    font-family: var(--colony-icon-font);
    font-weight: 900;
    font-size: var(--fs-18);
  }
  .ach__icon-fallback--locked { color: var(--colony-text-dimmer); }

  .ach__body { flex: 1; min-width: 0; }
  .ach__name { font-weight: var(--fw-bold); overflow-wrap: anywhere; }
  .ach__desc { font-size: var(--fs-11); margin-top: 2px; }
  .ach__api { font-size: var(--fs-10); margin-top: 4px; overflow-wrap: anywhere; }

  .ach__empty { text-align: center; padding: 40px; }
</style>
