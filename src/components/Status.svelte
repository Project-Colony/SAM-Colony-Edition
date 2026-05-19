<script lang="ts">
  import type { Info, StatusLine } from "../lib/types";

  let { info, log }: { info: Info; log: StatusLine[] } = $props();

  let headerUrl = $derived(
    info.app_id > 0
      ? `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${info.app_id}/header.jpg`
      : ""
  );
</script>

<section class="status">
  {#if info.app_id > 0}
    <img class="status__cover" src={headerUrl} alt="" />
  {/if}

  <div class="status__block">
    <div class="status__eyebrow">Game</div>
    <div class="status__value">{info.app_name || "—"}</div>
    <div class="status__sub dim">
      {info.app_id !== 0 ? `App ID ${info.app_id}` : ""}
    </div>
  </div>

  <div class="status__block">
    <div class="status__eyebrow">User</div>
    <div class="status__value">{info.user_name || "—"}</div>
    <div class="status__sub dim">
      {info.user_id !== 0 ? `SteamID ${info.user_id}` : ""}
    </div>
  </div>

  <div class="status__block status__log">
    <div class="status__eyebrow">Activity</div>
    <ul class="status__list">
      {#each log as line, i (i + "-" + line.msg)}
        <li class:dim={!line.fresh}>{line.msg}</li>
      {/each}
    </ul>
  </div>
</section>

<style>
  .status {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .status__cover {
    width: 100%;
    border-radius: var(--radius-card);
    border: 1px solid var(--red-soft);
    display: block;
  }
  .status__block { display: flex; flex-direction: column; gap: 1px; }
  .status__eyebrow {
    font-family: var(--font-display);
    font-size: var(--fs-xs);
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-display);
    text-transform: uppercase;
    color: var(--ink-dim);
  }
  .status__value {
    font-size: var(--fs-md);
    font-weight: var(--fw-semibold);
    color: var(--ink);
  }
  .status__sub { font-size: var(--fs-xs); }
  .status__log { flex: 1; min-height: 0; }
  .status__list {
    list-style: none;
    margin: 4px 0 0;
    padding: 0;
    font-size: var(--fs-sm);
    font-weight: var(--fw-regular);
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 180px;
    overflow: auto;
  }
</style>
