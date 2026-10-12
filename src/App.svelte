<script lang="ts">
  import { onMount } from "svelte";
  import { describeError, t } from "./i18n/index.svelte";
  import { GEAR } from "./lib/icons";
  import { cmd } from "./lib/tauri";
  import type { Achievement, Info, Message, Stat, StatusLine, View } from "./lib/types";
  import Search from "./components/Search.svelte";
  import SectionNav from "./components/SectionNav.svelte";
  import Status from "./components/Status.svelte";
  import AchievementView from "./components/AchievementView.svelte";
  import StatisticView from "./components/StatisticView.svelte";
  import Preferences from "./components/Preferences.svelte";

  let achievements: Achievement[] = $state([]);
  let icons: Record<string, string> = $state({});
  let stats: Stat[] = $state([]);
  let log: StatusLine[] = $state([]);
  let info: Info = $state({ app_id: 0, app_name: "", user_id: 0, user_name: "" });
  let view: View = $state("achievements");
  let databaseReady = $state(false);
  let preferencesOpen = $state(false);
  let nextLine = 0;

  function pushStatus(input: Message | Message[]) {
    const fresh = (Array.isArray(input) ? input : [input]).map((text) => ({
      id: nextLine++,
      text,
      fresh: true,
    }));
    log = [...fresh, ...log.map((l) => ({ ...l, fresh: false }))];
  }

  function selectSection(next: View) {
    view = next;
    preferencesOpen = false;
  }

  // Shortcuts by key position for the digits, so Ctrl+1 is the same key on
  // AZERTY; Ctrl+, by character, wherever the layout puts the comma.
  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && preferencesOpen) {
      preferencesOpen = false;
    } else if (!(event.ctrlKey || event.metaKey) || event.altKey || event.shiftKey) {
      return;
    } else if (event.key === ",") {
      preferencesOpen = !preferencesOpen;
    } else if (event.code === "Digit1" || event.code === "Numpad1") {
      selectSection("achievements");
    } else if (event.code === "Digit2" || event.code === "Numpad2") {
      selectSection("statistics");
    } else {
      return;
    }
    event.preventDefault();
  }

  async function loadAchievements(refresh: boolean) {
    try {
      achievements = await cmd.loadAchievements();
      if (refresh) pushStatus(() => t("log_achievements_loaded"));
    } catch (err) {
      pushStatus(() => t("log_load_achievements_failed", { error: describeError(err) }));
    }
  }

  async function loadAchievementIcons(appid: number) {
    try {
      icons = await cmd.loadAchievementIcons(appid);
    } catch (err) {
      pushStatus(() => t("log_load_icons_failed", { error: describeError(err) }));
    }
  }

  async function loadStatistics(appid: number, refresh: boolean) {
    try {
      stats = await cmd.loadStatistics(appid);
      if (refresh) pushStatus(() => t("log_statistics_loaded"));
    } catch (err) {
      pushStatus(() => t("log_load_statistics_failed", { error: describeError(err) }));
    }
  }

  async function updateUserInfo(appid: number, appname: string) {
    try {
      const user = await cmd.retrieveUser();
      info = {
        app_id: appid,
        app_name: appname,
        user_id: user.user_steam_id,
        user_name: user.user_name,
      };
    } catch (err) {
      pushStatus(() => t("log_retrieve_user_failed", { error: describeError(err) }));
    }
  }

  async function handleAppSelected(appid: number, name: string) {
    if (appid <= 0) return;
    try {
      await cmd.startClient(appid);
      pushStatus(() => t("log_starting_client"));
      await Promise.all([
        loadStatistics(appid, true),
        loadAchievements(true),
        loadAchievementIcons(appid),
        updateUserInfo(appid, name),
      ]);
    } catch (err) {
      // Rust returns a code for the reason: Steam not running, game not
      // owned, Steam out of date.
      pushStatus(() => describeError(err));
    }
  }

  onMount(async () => {
    pushStatus(() => t("log_loading_database"));
    try {
      const status = await cmd.fetchGames();
      pushStatus(() => t(status));
      databaseReady = true;
    } catch (err) {
      pushStatus(() => describeError(err));
    }
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="layout">
  <aside class="sidebar">
    <nav class="sidebar__nav">
      <!-- The program's name is the way into Preferences (navigation.md). -->
      <button
        class="identity"
        class:identity--open={preferencesOpen}
        aria-expanded={preferencesOpen}
        title={t("settings_title")}
        onclick={() => (preferencesOpen = !preferencesOpen)}
      >
        <span class="identity__name">{t("app_name")}</span>
        <span class="identity__gear glyph" aria-hidden="true">{GEAR}</span>
      </button>
      <span class="sidebar__label">{t("nav_sections")}</span>
      <SectionNav {view} onSelect={selectSection} />
      <span class="sidebar__hint">{t("nav_hint")}</span>
    </nav>

    <section class="sidebar__section">
      <Search onAppSelected={handleAppSelected} {databaseReady} />
    </section>

    <section class="sidebar__section sidebar__status">
      <Status {info} {log} />
    </section>
  </aside>

  <main class="main">
    {#if preferencesOpen}
      <Preferences onClose={() => (preferencesOpen = false)} onStatus={pushStatus} />
    {:else if info.app_id === 0}
      <div class="empty panel">
        <div class="eyebrow empty__eyebrow">{t("app_full_name")}</div>
        <h2 class="title empty__title">{t("empty_title")}</h2>
        <p class="muted">{t("empty_body")}</p>
      </div>
    {:else if view === "achievements"}
      <AchievementView
        {achievements}
        {icons}
        onStatus={pushStatus}
        onReload={loadAchievements}
      />
    {:else}
      <StatisticView
        {stats}
        onStatus={pushStatus}
        onReload={(refresh) => loadStatistics(info.app_id, refresh)}
      />
    {/if}
  </main>
</div>

<style>
  .layout {
    display: grid;
    /* Grows with the text so French at 1.68x still fits, but never takes
       the window from the content. */
    grid-template-columns: min(calc(280px * var(--font-scale)), 38vw) minmax(0, 1fr);
    height: 100vh;
  }

  .sidebar {
    background: var(--colony-bg-sidebar);
    border-right: 1px solid var(--colony-divider);
    display: flex;
    flex-direction: column;
    overflow: hidden auto;
  }
  .sidebar__nav {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px 14px 12px;
    border-bottom: 1px solid var(--colony-divider);
  }

  .identity {
    align-self: flex-start;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 8px;
    margin-bottom: 8px;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: var(--colony-text-primary);
    cursor: pointer;
  }
  .identity:hover { background: var(--colony-bg-card-hover); }
  .identity--open { background: var(--colony-bg-selected); }
  .identity__name {
    font-size: var(--fs-30);
    font-weight: var(--fw-bold);
    line-height: 1;
  }
  .identity__gear {
    font-size: var(--fs-14);
    color: var(--colony-text-dimmer);
  }
  .identity--open .identity__gear { color: var(--colony-text-primary); }

  .sidebar__label {
    font-size: var(--fs-13);
    color: var(--colony-text-muted);
  }
  .sidebar__hint {
    font-size: var(--fs-10);
    color: var(--colony-text-dimmest);
  }

  .sidebar__section {
    padding: 12px 14px;
    border-bottom: 1px solid var(--colony-divider);
  }
  .sidebar__status {
    flex: 1;
    border-bottom: none;
  }

  .main {
    padding: 16px 22px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }

  .empty {
    margin: auto;
    max-width: calc(480px * var(--font-scale));
    text-align: center;
    padding: 28px;
  }
  .empty__eyebrow { margin-bottom: 10px; }
  .empty__title {
    font-size: var(--fs-18);
    margin-bottom: 10px;
  }
</style>
