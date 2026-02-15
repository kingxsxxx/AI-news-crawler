import { invoke } from "@tauri-apps/api/core";
import type { Article, CrawlResult, ListResponse, Settings } from "../types";

export const api = {
  health: () => invoke<string>("health"),
  openExternal: (url: string) => invoke<void>("open_external", { url }),
  runCrawler: () => invoke<CrawlResult>("crawler_run_once"),
  regenerateSummaries: () => invoke<number>("articles_regenerate_summaries"),
  listArticles: (page = 1, pageSize = 20, category?: string) =>
    invoke<ListResponse>("articles_list", {
      query: { page, page_size: pageSize, category },
    }),
  searchArticles: (keyword: string) =>
    invoke<Article[]>("search_query", { payload: { keyword } }),
  toggleBookmark: (id: string, value: boolean) =>
    invoke<void>("article_bookmark", { payload: { id, value } }),
  toggleRead: (id: string, value: boolean) =>
    invoke<void>("article_mark_read", { payload: { id, value } }),
  manualAdd: (url: string) => invoke<Article>("manual_add", { payload: { url } }),
  summarize: (content: string) => invoke<string>("ai_summarize", { content }),
  getSettings: () => invoke<Settings>("settings_get"),
  updateSettings: (payload: Settings) => invoke<Settings>("settings_update", { payload }),
};
