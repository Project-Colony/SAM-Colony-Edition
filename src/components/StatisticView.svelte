<script lang="ts">
  import { cmd } from "../lib/tauri";
  import type { Stat } from "../lib/types";

  let {
    stats,
    onStatus,
    onReload,
  }: {
    stats: Stat[];
    onStatus: (msg: string | string[]) => void;
    onReload: (refresh: boolean) => void;
  } = $props();

  let items: (Stat & { _draft: number })[] = $state([]);

  $effect(() => {
    items = $state.snapshot(stats).map((s) => ({ ...s, _draft: s.value }));
  });

  async function apply() {
    const alerts: string[] = [];
    for (const it of items) {
      if (it._draft === it.value || Number.isNaN(it._draft)) continue;
      try {
        await cmd.commitStatistic(it.api_name, it._draft);
        alerts.push(`${it.name || it.api_name} set to ${it._draft}`);
      } catch (err) {
        alerts.push(`Failed: ${it.api_name} (${String(err)})`);
      }
    }
    try {
      await cmd.storeStats();
      onStatus(alerts.length > 0 ? alerts : "No statistic changes to commit");
      onReload(false);
    } catch (err) {
      onStatus(`Store stats failed: ${String(err)}`);
    }
  }

  function reset() {
    onReload(true);
  }
</script>

<div class="stats">
  {#if items.length === 0}
    <div class="panel stats__empty">
      <p class="muted">No statistics were found for this game.</p>
    </div>
  {:else}
    <ul class="stats__list">
      {#each items as item, idx (item.api_name)}
        <li class="card stats__row">
          <div class="stats__label">
            <div class="stats__name">{item.name.length > 0 ? item.name : item.api_name}</div>
            <div class="stats__api dim">{item.api_name}</div>
          </div>
          <input
            class="input stats__input"
            type="number"
            min={item.min}
            max={item.max}
            bind:value={items[idx]._draft}
          />
          <div class="stats__bounds dim">
            {item.min} <span>—</span> {item.max}
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <footer class="stats__actions">
    <button class="btn" onclick={reset}>Refresh</button>
    <button class="btn btn--primary" onclick={apply}>Apply</button>
  </footer>
</div>

<style>
  .stats { display: flex; flex-direction: column; gap: 10px; flex: 1; min-height: 0; }
  .stats__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow: auto;
    min-height: 0;
  }
  .stats__row {
    display: grid;
    grid-template-columns: 1fr 140px auto;
    gap: 14px;
    align-items: center;
  }
  .stats__label { min-width: 0; }
  .stats__name { font-weight: var(--fw-semibold); }
  .stats__api { font-size: var(--fs-xs); font-style: italic; font-weight: var(--fw-light); }
  .stats__input { text-align: right; font-variant-numeric: tabular-nums; }
  .stats__bounds { font-size: var(--fs-sm); font-variant-numeric: tabular-nums; }

  .stats__empty { text-align: center; padding: 40px; }

  .stats__actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding-top: 8px;
    border-top: 1px solid var(--red-very-soft);
  }
</style>
