export type Article = {
  id: string;
  title: string;
  summary: string;
  content: string;
  url: string;
  source: string;
  category: string;
  published_at: string;
  fetched_at: string;
  heat_score: number;
  is_read: boolean;
  is_bookmarked: boolean;
  image_url: string;
};

export type Settings = {
  theme: string;
  ai_model: string;
  ai_base_url: string;
  ai_api_key: string;
  ai_summary_enabled: boolean;
};

export type ListResponse = {
  items: Article[];
  total: number;
  page: number;
  page_size: number;
};

export type CrawlResult = {
  inserted: number;
  failed_sources: number;
};

export type SummaryUpdateStatus = {
  isRunning: boolean;
  total: number | null;
  current: number;
  updated: number;
  currentTitle: string | null;
  error: string | null;
};
