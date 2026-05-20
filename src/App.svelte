<script lang="ts">
  import { onMount } from "svelte";
  import { cmd } from "./lib/tauri";
  import type { Achievement, Info, Stat, StatusLine, View } from "./lib/types";
  import Search from "./components/Search.svelte";
  import Tabs from "./components/Tabs.svelte";
  import Status from "./components/Status.svelte";
  import AchievementView from "./components/AchievementView.svelte";
  import StatisticView from "./components/StatisticView.svelte";

  let achievements: Achievement[] = $state([]);
  let icons: Record<string, string> = $state({});
  let stats: Stat[] = $state([]);
  let log: StatusLine[] = $state([]);
  let info: Info = $state({ app_id: 0, app_name: "", user_id: 0, user_name: "" });
  let view: View = $state("achievements");
  let databaseReady = $state(false);

  function pushStatus(input: string | string[]) {
    const fresh: StatusLine[] = Array.isArray(input)
      ? input.map((msg) => ({ msg, fresh: true }))
      : [{ msg: input, fresh: true }];
    log = [...fresh, ...log.map((l) => ({ ...l, fresh: false }))];
  }

  async function loadAchievements(refresh: boolean) {
    try {
      achievements = await cmd.loadAchievements();
      if (refresh) pushStatus("Achievements loaded");
    } catch (err) {
      pushStatus(`Load achievements failed: ${String(err)}`);
    }
  }

  async function loadAchievementIcons(appid: number) {
    try {
      icons = await cmd.loadAchievementIcons(appid);
    } catch (err) {
      pushStatus(`Load icons failed: ${String(err)}`);
    }
  }

  async function loadStatistics(appid: number, refresh: boolean) {
    try {
      stats = await cmd.loadStatistics(appid);
      if (refresh) pushStatus("Statistics loaded");
    } catch (err) {
      pushStatus(`Load statistics failed: ${String(err)}`);
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
      pushStatus(`Retrieve user failed: ${String(err)}`);
    }
  }

  async function handleAppSelected(appid: number, name: string) {
    if (appid <= 0) return;
    try {
      await cmd.startClient(appid);
      pushStatus("Starting client.");
      await Promise.all([
        loadStatistics(appid, true),
        loadAchievements(true),
        loadAchievementIcons(appid),
        updateUserInfo(appid, name),
      ]);
    } catch (err) {
      // Rust returns a user-facing reason: "You don't own this game...",
      // "Steam is not running...", "Steam client is out of date...".
      pushStatus(String(err));
    }
  }

  onMount(async () => {
    pushStatus("Loading database...");
    try {
      const status = await cmd.fetchGames();
      pushStatus(status);
      databaseReady = true;
    } catch (err) {
      pushStatus(String(err));
    }
  });
</script>

<div class="layout">
  <aside class="sidebar">
    <header class="brand">
      <h1 class="title brand__name">SAM</h1>
      <span class="brand__sub">Colony Edition</span>
    </header>

    <section class="sidebar__section">
      <Search onAppSelected={handleAppSelected} {databaseReady} />
    </section>

    <section class="sidebar__section">
      <Tabs bind:view />
    </section>

    <section class="sidebar__section sidebar__status">
      <Status {info} {log} />
    </section>
  </aside>

  <main class="main">
    {#if info.app_id === 0}
      <div class="empty panel">
        <div class="empty__eyebrow">SAM — Colony Edition</div>
        <h2 class="title empty__title">Search for a game to begin</h2>
        <p class="muted">
          Steam must be running and signed in. Type a name (≥2 characters) or paste an App ID.
        </p>
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
    grid-template-columns: 300px 1fr;
    height: 100vh;
  }

  .sidebar {
    background: var(--parchment-soft);
    border-right: var(--border-panel) solid var(--red);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .brand {
    padding: 18px 18px 14px;
    border-bottom: 1px solid var(--red-soft);
  }
  .brand__name {
    font-size: var(--fs-3xl);
    font-weight: var(--fw-bold);
    line-height: 1;
    letter-spacing: var(--tracking-tight);
  }
  .brand__sub {
    font-family: var(--font-display);
    font-size: var(--fs-xs);
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-display);
    text-transform: uppercase;
    display: block;
    margin-top: 4px;
    color: var(--ink-dim);
  }

  .sidebar__section {
    padding: 12px 14px;
    border-bottom: 1px solid var(--red-very-soft);
  }
  .sidebar__status {
    flex: 1;
    overflow: auto;
    border-bottom: none;
  }

  .main {
    padding: 16px 22px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .empty {
    margin: auto;
    max-width: 480px;
    text-align: center;
    padding: 28px;
  }
  .empty__eyebrow {
    font-family: var(--font-display);
    font-size: var(--fs-xs);
    font-weight: var(--fw-semibold);
    letter-spacing: var(--tracking-display);
    text-transform: uppercase;
    color: var(--red);
    margin-bottom: 10px;
  }
  .empty__title {
    font-size: var(--fs-xl);
    margin-bottom: 10px;
  }
</style>
