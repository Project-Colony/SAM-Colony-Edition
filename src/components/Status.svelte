<script lang="ts">
  import { t } from "../i18n/index.svelte";
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
    <div class="eyebrow">{t("status_game")}</div>
    <div class="status__value">{info.app_name || "-"}</div>
    <div class="status__sub dim">
      {info.app_id !== 0 ? t("status_app_id", { id: info.app_id }) : ""}
    </div>
  </div>

  <div class="status__block">
    <div class="eyebrow">{t("status_user")}</div>
    <div class="status__value">{info.user_name || "-"}</div>
    <div class="status__sub dim">
      {info.user_id !== 0 ? t("status_steam_id", { id: info.user_id }) : ""}
    </div>
  </div>

  <div class="status__block status__log">
    <div class="eyebrow">{t("status_activity")}</div>
    <ul class="status__list">
      {#each log as line (line.id)}
        <li class:dim={!line.fresh}>{line.text()}</li>
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
    border: 1px solid var(--colony-border-subtle);
    display: block;
  }
  .status__block { display: flex; flex-direction: column; gap: 1px; }
  .status__value {
    font-weight: var(--fw-bold);
    color: var(--colony-text-primary);
    overflow-wrap: anywhere;
  }
  .status__sub { font-size: var(--fs-10); }
  .status__log { flex: 1; min-height: 0; }
  .status__list {
    list-style: none;
    margin: 4px 0 0;
    padding: 0;
    font-size: var(--fs-11);
    color: var(--colony-text-secondary);
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: calc(180px * var(--font-scale));
    overflow: auto;
    overflow-wrap: anywhere;
  }
</style>
