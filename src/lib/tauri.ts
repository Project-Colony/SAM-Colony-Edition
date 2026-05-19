import { invoke } from "@tauri-apps/api/core";
import type { Achievement, App, Stat } from "./types";

export const cmd = {
  fetchGames: () => invoke<string>("cmd_fetch_games"),
  searchName: (query: string) => invoke<App[]>("cmd_search_name", { query }),
  requestAppName: (appid: number) => invoke<string>("cmd_request_app_name", { appid }),
  startClient: (appid: number) => invoke<boolean>("cmd_start_client", { appid }),
  loadAchievements: () => invoke<Achievement[]>("cmd_load_achievements"),
  loadAchievementIcons: (appid: number) =>
    invoke<Record<string, string>>("cmd_load_achievement_icons", { appid }),
  loadStatistics: (appid: number) => invoke<Stat[]>("cmd_load_statistics", { appid }),
  commitAchievement: (name: string, unlocked: boolean) =>
    invoke<void>("cmd_commit_achievement", { name, unlocked }),
  commitStatistic: (name: string, value: number) =>
    invoke<void>("cmd_commit_statistics", { name, value }),
  storeStats: () => invoke<void>("cmd_store_stats"),
  retrieveUser: () =>
    invoke<{ user_steam_id: number; user_name: string }>("cmd_retrieve_user"),
};
