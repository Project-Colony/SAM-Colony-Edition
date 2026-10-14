<script lang="ts">
  import { describeError, t } from "../i18n/index.svelte";
  import { cmd } from "../lib/tauri";
  import type { Message, Stat } from "../lib/types";

  let {
    stats,
    onStatus,
    onReload,
  }: {
    stats: Stat[];
    onStatus: (msg: Message | Message[]) => void;
    onReload: (refresh: boolean) => void;
  } = $props();

  let items: (Stat & { _draft: number })[] = $state([]);

  $effect(() => {
    items = $state.snapshot(stats).map((s) => ({ ...s, _draft: s.value }));
  });

  async function apply() {
    const alerts: Message[] = [];
    let written = 0;
    for (const it of items) {
      if (it._draft === it.value || Number.isNaN(it._draft)) continue;
      try {
        await cmd.commitStatistic(it.api_name, it._draft);
        written++;
        const [name, value] = [it.name || it.api_name, it._draft];
        alerts.push(() => t("log_statistic_set", { name, value }));
      } catch (err) {
        const name = it.api_name;
        alerts.push(() => t("log_write_failed", { name, error: describeError(err) }));
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

  function reset() {
    onReload(true);
  }
</script>

<div class="stats">
  {#if items.length === 0}
    <div class="panel stats__empty">
      <p class="muted">{t("statistics_none")}</p>
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
            {item.min} <span>-</span> {item.max}
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <footer class="stats__actions">
    <button class="btn" onclick={reset}>{t("action_refresh")}</button>
    <button class="btn btn--primary" onclick={apply}>{t("action_apply")}</button>
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
    grid-template-columns: 1fr calc(140px * var(--font-scale)) auto;
    gap: 14px;
    align-items: center;
  }
  .stats__label { min-width: 0; }
  .stats__name { font-weight: var(--fw-bold); overflow-wrap: anywhere; }
  .stats__api { font-size: var(--fs-10); overflow-wrap: anywhere; }
  .stats__input { text-align: right; font-variant-numeric: tabular-nums; }
  .stats__bounds { font-size: var(--fs-11); font-variant-numeric: tabular-nums; }

  .stats__empty { text-align: center; padding: 40px; }

  .stats__actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding-top: 8px;
    border-top: 1px solid var(--colony-divider);
  }
</style>
