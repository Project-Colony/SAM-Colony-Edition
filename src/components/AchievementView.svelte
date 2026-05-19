<script lang="ts">
  import { cmd } from "../lib/tauri";
  import type { Achievement } from "../lib/types";

  let {
    achievements,
    icons,
    onStatus,
    onReload,
  }: {
    achievements: Achievement[];
    icons: Record<string, string>;
    onStatus: (msg: string | string[]) => void;
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
    const alerts: string[] = [];
    for (const it of items) {
      const original = achievements.find((a) => a.api_name === it.api_name);
      if (original && it.status !== original.status) {
        try {
          await cmd.commitAchievement(it.api_name, it.status);
          alerts.push(`${it.name} ${it.status ? "unlocked" : "locked"}`);
        } catch (err) {
          alerts.push(`Failed: ${it.name} (${String(err)})`);
        }
      }
    }
    try {
      await cmd.storeStats();
      onStatus(alerts.length > 0 ? alerts : "No changes to commit");
      onReload(false);
    } catch (err) {
      onStatus(`Store stats failed: ${String(err)}`);
    }
  }

  let unlockedCount = $derived(items.filter((a) => a.status).length);
</script>

<div class="ach">
  <header class="ach__toolbar panel">
    <input
      class="input ach__filter"
      type="text"
      placeholder="Filter by name or description"
      oninput={handleFilter}
    />
    <div class="ach__count">
      <span class="count__num">{unlockedCount}</span>
      <span class="count__sep dim">/</span>
      <span class="count__total dim">{items.length}</span>
    </div>
    <div class="ach__buttons">
      <button class="btn" onclick={() => onReload(true)}>Refresh</button>
      <button class="btn btn--primary" onclick={apply}>Apply changes</button>
    </div>
  </header>

  <nav class="ach__subnav">
    <button class="btn" onclick={() => selectAll(true)}>Select all</button>
    <button class="btn" onclick={() => selectAll(false)}>Deselect all</button>
    <button class="btn" onclick={() => showLockedOnly(true)}>Unlocked only</button>
    <button class="btn" onclick={() => showLockedOnly(false)}>Locked only</button>
  </nav>

  {#if items.length === 0}
    <div class="panel ach__empty">
      <p class="muted">No achievements found.</p>
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
                <div class="ach__icon-fallback" class:ach__icon-fallback--locked={!item.status}>
                  {item.status ? "✦" : "·"}
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
    grid-template-columns: 1fr auto auto;
    gap: 12px;
    align-items: center;
    padding: 10px 14px;
  }
  .ach__filter { max-width: 360px; }
  .ach__count { font-family: var(--font-display); display: flex; align-items: baseline; gap: 4px; }
  .count__num { font-size: var(--fs-xl); font-weight: var(--fw-bold); color: var(--red); }
  .count__sep, .count__total { font-size: var(--fs-lg); font-weight: var(--fw-light); }
  .ach__buttons { display: flex; gap: 6px; }

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

  .ach__checkbox {
    appearance: none;
    width: 18px; height: 18px;
    border: var(--border-thin) solid var(--red);
    border-radius: 3px;
    background: var(--parchment);
    cursor: pointer;
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }
  .ach__checkbox:checked {
    background: var(--red);
  }
  .ach__checkbox:checked::after {
    content: "✓";
    color: var(--parchment);
    font-weight: var(--fw-bold);
    font-size: 13px;
    line-height: 1;
  }

  .ach__icon-wrap {
    width: 48px;
    height: 48px;
    border: 1px solid var(--red-soft);
    border-radius: var(--radius-input);
    overflow: hidden;
    background: var(--parchment);
    flex-shrink: 0;
    display: grid;
    place-items: center;
  }
  .ach__icon { width: 100%; height: 100%; object-fit: cover; display: block; }
  .ach__icon-fallback {
    color: var(--red);
    font-family: var(--font-display);
    font-size: var(--fs-xl);
    font-weight: var(--fw-bold);
  }
  .ach__icon-fallback--locked {
    color: var(--ink-very-dim);
    font-weight: var(--fw-light);
  }

  .ach__body { flex: 1; min-width: 0; }
  .ach__name {
    font-family: var(--font-display);
    font-size: var(--fs-md);
    font-weight: var(--fw-semibold);
  }
  .ach__desc { font-size: var(--fs-sm); margin-top: 2px; }
  .ach__api {
    font-size: var(--fs-xs);
    font-style: italic;
    font-weight: var(--fw-light);
    margin-top: 4px;
  }

  .ach__empty { text-align: center; padding: 40px; }
</style>
