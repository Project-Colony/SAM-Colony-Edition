<script lang="ts">
  import { cmd } from "../lib/tauri";
  import type { App } from "../lib/types";

  let {
    onAppSelected,
    databaseReady = true,
  }: {
    onAppSelected: (id: number, name: string) => void;
    databaseReady?: boolean;
  } = $props();

  let query = $state("");
  let applist: App[] = $state([]);
  let active = $state(false);
  let searched = $state(false);
  let inFlight = $state(false);
  let debounceTimer: number | null = null;

  function handleChange(event: Event) {
    query = (event.target as HTMLInputElement).value;

    if (debounceTimer !== null) clearTimeout(debounceTimer);
    debounceTimer = window.setTimeout(() => {
      void runSearch(query);
    }, 750);
  }

  async function runSearch(value: string) {
    if (value.length < 2) {
      applist = [];
      searched = false;
      return;
    }
    inFlight = true;
    try {
      applist = await cmd.searchName(value);
      searched = true;
    } catch (err) {
      console.error("search failed", err);
    } finally {
      inFlight = false;
    }
  }

  // If the user typed before the games database finished loading, the first
  // search returned []. Auto-retry once the database becomes ready.
  $effect(() => {
    if (databaseReady && query.trim().length >= 2 && applist.length === 0) {
      void runSearch(query);
    }
  });

  function pick(app: App) {
    onAppSelected(app.appid, app.name);
    active = false;
    query = app.name;
    applist = [];
    searched = false;
  }

  async function manualLaunch(id: number) {
    try {
      const name = await cmd.requestAppName(id);
      onAppSelected(id, name);
      active = false;
    } catch (err) {
      console.error("manual launch failed", err);
    }
  }

  function handleBlur(e: FocusEvent) {
    // Defer so click on a dropdown item still fires
    setTimeout(() => {
      const next = document.activeElement;
      if (!(e.currentTarget as HTMLElement)?.contains(next)) active = false;
    }, 120);
  }

  const queryAsNumber = $derived(Number(query));
  const queryIsNumeric = $derived(!Number.isNaN(queryAsNumber) && query.length > 0);
  // Show the dropdown whenever there is meaningful state to convey: results,
  // a numeric AppID launch shortcut, an in-flight search, a confirmed empty
  // result, or a "database still loading" hint.
  const hasState = $derived(
    applist.length > 0 ||
      queryIsNumeric ||
      inFlight ||
      (searched && applist.length === 0) ||
      (!databaseReady && query.trim().length >= 2)
  );
</script>

<div
  class="search"
  role="search"
  onfocusin={() => (active = true)}
  onfocusout={handleBlur}
>
  <input
    class="input"
    type="text"
    placeholder="Search by name or App ID"
    value={query}
    oninput={handleChange}
  />

  {#if active && hasState}
    <ul class="dropdown">
      {#if !databaseReady && query.trim().length >= 2}
        <li class="dropdown__hint dim">Loading database, retry in a moment…</li>
      {:else if inFlight}
        <li class="dropdown__hint dim">Searching…</li>
      {:else}
        {#if queryIsNumeric}
          <li>
            <button class="dropdown__item dropdown__item--manual" onclick={() => manualLaunch(queryAsNumber)}>
              <span class="dropdown__lead">Launch App ID</span>
              <span class="dropdown__id">{queryAsNumber}</span>
            </button>
          </li>
        {/if}
        {#each applist as app (app.appid)}
          <li>
            <button class="dropdown__item" onclick={() => pick(app)}>
              <span class="dropdown__name">{app.name}</span>
              <span class="dropdown__id dim">{app.appid}</span>
            </button>
          </li>
        {/each}
        {#if searched && applist.length === 0 && !queryIsNumeric}
          <li class="dropdown__hint dim">No games found for "{query}"</li>
        {/if}
      {/if}
    </ul>
  {/if}
</div>

<style>
  .search { position: relative; }
  .dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 10;
    list-style: none;
    margin: 0;
    padding: 4px;
    background: var(--parchment);
    border: var(--border-thin) solid var(--red);
    border-radius: var(--radius-card);
    box-shadow: 0 4px 12px var(--parchment-shadow);
    max-height: 320px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .dropdown__item {
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-input);
    padding: 6px 8px;
    font-family: var(--font-body);
    font-size: var(--fs-md);
    font-weight: var(--fw-regular);
    color: var(--ink);
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .dropdown__item:hover {
    background: var(--parchment-hover);
    border-color: var(--red-soft);
  }
  .dropdown__item--manual { font-weight: var(--fw-semibold); }
  .dropdown__lead { color: var(--red); }
  .dropdown__name { font-weight: var(--fw-medium); }
  .dropdown__id { font-size: var(--fs-sm); font-weight: var(--fw-light); }
  .dropdown__hint {
    padding: 8px 10px;
    font-size: var(--fs-sm);
    font-style: italic;
    text-align: center;
  }
</style>
