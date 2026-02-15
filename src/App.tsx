import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "./lib/api";
import type { Article, Settings, SummaryUpdateStatus } from "./types";
import { ArticleList } from "./components/article/ArticleList";
import { SummaryUpdateProgress } from "./components/settings/SummaryUpdateProgress";

type Tab = "search" | "saved" | "settings";
type UiState = "idle" | "loading" | "success" | "error";

const tabs: Tab[] = ["search", "saved", "settings"];

// 分类筛选选项
const categoryOptions = [
  { value: "all", label: "全部", icon: "📰" },
  { value: "AI", label: "AI 资讯", icon: "🤖" },
  { value: "GitHub", label: "开源项目", icon: "💻" },
  { value: "Tech", label: "科技资讯", icon: "📱" },
];

export default function App(): JSX.Element {
  const [tab, setTab] = useState<Tab>("search");
  const [articles, setArticles] = useState<Article[]>([]);
  const [searchKeyword, setSearchKeyword] = useState("");
  const [uiState, setUiState] = useState<UiState>("idle");
  const [statusMessage, setStatusMessage] = useState("准备就绪");
  const [manualUrl, setManualUrl] = useState("");
  const [summaryInput, setSummaryInput] = useState("");
  const [summaryOutput, setSummaryOutput] = useState("");
  const [lastUpdated, setLastUpdated] = useState<string>("-");
  const [settings, setSettings] = useState<Settings>({
    theme: "auto",
    ai_model: "qwen3-max",
    ai_base_url: "",
    ai_api_key: "",
    ai_summary_enabled: true,
  });

  // 分类筛选状态（用于 SEARCH 栏）
  const [searchCategory, setSearchCategory] = useState<string>("all");

  // 分页状态
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [totalArticles, setTotalArticles] = useState(0);
  const articlesPerPage = 50; // 每页50篇文章

  // 计算总页数
  const calculatedTotalPages = Math.max(1, Math.ceil(totalArticles / articlesPerPage));

  // Toast 消息状态
  const [toast, setToast] = useState<{ message: string; type: "success" | "error" | "warning" } | null>(null);

  // Batch update progress state
  const [updateProgress, setUpdateProgress] = useState<SummaryUpdateStatus>({
    isRunning: false,
    total: null,
    current: 0,
    updated: 0,
    currentTitle: null,
    error: null,
  });

  const loading = uiState === "loading";

  const bookmarked = useMemo(
    () => articles.filter((article) => article.is_bookmarked),
    [articles]
  );

  // 主题切换效果
  useEffect(() => {
    const root = document.documentElement;
    root.classList.remove("light", "dark", "auto");

    if (settings.theme === "auto") {
      root.classList.add("auto");
    } else {
      root.classList.add(settings.theme);
    }
  }, [settings.theme]);

  // 显示 Toast 消息
  const showToast = (message: string, type: "success" | "error" | "warning" = "success"): void => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  const setSuccess = (msg: string): void => {
    setUiState("success");
    setStatusMessage(msg);
    setLastUpdated(new Date().toLocaleString("zh-CN"));
    showToast(msg, "success");
  };

  const setError = (msg: string): void => {
    setUiState("error");
    setStatusMessage(msg);
    showToast(msg, "error");
  };

  const loadArticles = async (page: number = 1, showMessage = true): Promise<void> => {
    setUiState("loading");
    try {
      // 根据分类筛选请求文章
      const category = searchCategory === "all" ? undefined : searchCategory;
      const res = await api.listArticles(
        page,
        articlesPerPage,
        category
      );

      setTotalArticles(res.total);
      setArticles(res.items);
      setCurrentPage(page);
      setTotalPages(calculatedTotalPages);

      if (showMessage) {
        const categoryLabel = searchCategory === "all" ? "全部" : categoryOptions.find(c => c.value === searchCategory)?.label || searchCategory;
        setSuccess(`${categoryLabel} - 共 ${res.total} 篇文章`);
      }
    } catch (error) {
      setError(`加载失败: ${String(error)}`);
    } finally {
      setUiState("idle");
    }
  };

  const goToPage = (page: number): void => {
    if (page >= 1 && page <= calculatedTotalPages) {
      void loadArticles(page, false);
    }
  };

  useEffect(() => {
    // 初始加载时不显示提示消息
    void loadArticles(1, false).catch((error) => {
      console.error("Failed to load articles on mount:", error);
    });
    api
      .getSettings()
      .then(setSettings)
      .catch((error) => {
        console.error("Failed to load settings:", error);
      });
  }, []);

  // 分类改变时重新加载文章
  useEffect(() => {
    void loadArticles(1, false);
  }, [searchCategory]);

  // 批量更新进度 - 使用事件监听替代轮询
  useEffect(() => {
    let mounted = true;

    const setupListeners = async () => {
      try {
        const unlisteners = await Promise.all([
          listen<{ total: number }>('app://summaries-update:start', (event) => {
            if (!mounted) return;
            setUpdateProgress({
              isRunning: true,
              total: event.payload.total,
              current: 0,
              updated: 0,
              currentTitle: null,
              error: null,
            });
          }),

          listen<{ current: number; total: number; title: string; updated: number }>(
            'app://summaries-update:progress',
            (event) => {
              if (!mounted) return;
              setUpdateProgress((prev) => ({
                ...prev,
                current: event.payload.current,
                total: event.payload.total,
                currentTitle: event.payload.title,
                updated: event.payload.updated,
              }));
            }
          ),

          listen<{ total_updated: number; total_processed: number }>(
            'app://summaries-update:complete',
            (event) => {
              if (!mounted) return;
              setUpdateProgress((prev) => ({
                ...prev,
                isRunning: false,
              }));
              setSuccess(`批量更新完成！成功更新 ${event.payload.total_updated} 篇文章`);
              void loadArticles(1, false);
            }
          ),
        ]);

        return () => {
          unlisteners.forEach((unlisten) => unlisten());
        };
      } catch (error) {
        console.warn('Failed to setup event listeners:', error);
        return () => {};
      }
    };

    const cleanupPromise = setupListeners();

    return () => {
      mounted = false;
      cleanupPromise.then((cleanup) => cleanup());
    };
  }, []);

  const runCrawler = async (): Promise<void> => {
    setUiState("loading");
    try {
      // 先尝试运行爬虫，设置超时保护（60秒，因为不再等待AI摘要）
      const timeoutPromise = new Promise<{ inserted: number; failed_sources: number }>((_, reject) =>
        setTimeout(() => reject(new Error("爬虫超时，请检查网络连接")), 60000)
      );

      const result = await Promise.race([
        api.runCrawler(),
        timeoutPromise,
      ]);

      // 抓取完成后加载文章
      await loadArticles(1, false);

      // 只显示新增文章数
      if (result.inserted > 0) {
        setSuccess(`新增 ${result.inserted} 篇文章`);
      } else if (result.failed_sources > 0) {
        setSuccess(`没有新文章，${result.failed_sources} 个源失败`);
      } else {
        setSuccess("没有新文章");
      }
    } catch (error) {
      console.error("Crawler error:", error);
      setError(`抓取失败: ${String(error)}`);
      // 即使爬虫失败，也尝试加载已有文章（不显示提示）
      try {
        await loadArticles(1, false);
      } catch (loadError) {
        console.error("Load articles error:", loadError);
      }
    } finally {
      setUiState("idle");
    }
  };

  const onSearch = async (): Promise<void> => {
    if (!searchKeyword.trim()) {
      await loadArticles(1, false);
      return;
    }
    setUiState("loading");
    try {
      const list = await api.searchArticles(searchKeyword.trim());
      setArticles(list);
      setTotalArticles(list.length);
      setCurrentPage(1);
      setTotalPages(1); // 搜索结果只显示一页
      setSuccess(`搜索到 ${list.length} 篇文章`);
    } catch (error) {
      setError(`搜索失败: ${String(error)}`);
    } finally {
      setUiState("idle");
    }
  };

  const onManualAdd = async (): Promise<void> => {
    if (!manualUrl.trim()) return;
    setUiState("loading");
    try {
      await api.manualAdd(manualUrl.trim());
      setManualUrl("");
      await loadArticles(currentPage, false);
      setSuccess("已添加文章");
    } catch (error) {
      setError(`添加失败: ${String(error)}`);
    } finally {
      setUiState("idle");
    }
  };

  const toggleBookmark = async (id: string, value: boolean): Promise<void> => {
    try {
      await api.toggleBookmark(id, value);
      setArticles((prev) =>
        prev.map((item) => (item.id === id ? { ...item, is_bookmarked: value } : item))
      );
      setSuccess(value ? "已收藏" : "已取消收藏");
    } catch (error) {
      setError(`操作失败: ${String(error)}`);
    }
  };

  const summarize = async (): Promise<void> => {
    if (!summaryInput.trim()) return;
    setUiState("loading");
    try {
      const output = await api.summarize(summaryInput);
      setSummaryOutput(output);
      setSuccess("AI 摘要生成完成");
    } catch (error) {
      setSummaryOutput("");
      setError(`AI 摘要失败: ${String(error)}`);
    } finally {
      setUiState("idle");
    }
  };

  const saveSettings = async (): Promise<void> => {
    setUiState("loading");
    try {
      const next = await api.updateSettings(settings);
      setSettings(next);
      setSuccess("设置已保存");
    } catch (error) {
      setError(`保存设置失败: ${String(error)}`);
    } finally {
      setUiState("idle");
    }
  };

  const listItems = tab === "saved" ? bookmarked : articles;

  // 计算今日新增文章数
  const todayCount = useMemo(() => {
    const today = new Date().toDateString();
    return articles.filter((a) => new Date(a.fetched_at).toDateString() === today).length;
  }, [articles]);

  return (
    <main className="app-shell">
      <aside className="panel sidebar">
        <div className="brand">
          <h1>AI RESOURCES APP</h1>
        </div>
        <nav className="tab-list">
          {tabs.map((item) => (
            <button
              key={item}
              className={`tab-btn ${tab === item ? "active" : ""}`}
              onClick={() => setTab(item)}
              type="button"
            >
              {item === "search" ? "🔍" : item === "saved" ? "⭐" : "⚙️"} {item.toUpperCase()}
            </button>
          ))}
        </nav>
        <button
          className={`primary-btn ${loading ? "loading" : ""}`}
          onClick={runCrawler}
          type="button"
          disabled={loading}
        >
          {loading ? (
            <span className="loading-spinner">
              <span className="spinner"></span>
              <span className="loading-text">LOADING...</span>
            </span>
          ) : (
            "REFRESH"
          )}
        </button>
      </aside>

      <section className="panel content-panel">
        <header className="topbar">
          <input
            value={searchKeyword}
            onChange={(event) => setSearchKeyword(event.target.value)}
            placeholder="搜索标题或摘要..."
            onKeyDown={(event) => event.key === "Enter" && void onSearch()}
          />
          <button type="button" className="btn-secondary" onClick={onSearch} disabled={loading}>
            🔍 搜索
          </button>
        </header>

        {tab === "search" && (
          <>
            {/* 分类筛选器 - 使用 feed 分类 */}
            <div className="category-filter">
              {categoryOptions.map((cat) => (
                <button
                  key={cat.value}
                  type="button"
                  className={`category-chip ${searchCategory === cat.value ? "active" : ""}`}
                  onClick={() => {
                    setSearchCategory(cat.value);
                    setCurrentPage(1);
                    // 如果有搜索词，清空它以显示分类结果
                    if (searchKeyword) {
                      setSearchKeyword("");
                    }
                  }}
                >
                  {cat.icon} {cat.label}
                </button>
              ))}
            </div>

            {/* 手动添加链接 */}
            <section className="manual-add">
              <input
                value={manualUrl}
                onChange={(event) => setManualUrl(event.target.value)}
                placeholder="粘贴文章链接手动添加..."
              />
              <button type="button" className="btn-primary" onClick={onManualAdd} disabled={loading}>
                ➕ 添加链接
              </button>
            </section>
          </>
        )}

        {tab !== "settings" && (
          <>
            <ArticleList
              items={listItems}
              onToggleBookmark={toggleBookmark}
            onRefresh={tab === "search" ? runCrawler : undefined}
            emptyMessage={
              tab === "saved"
                ? "暂无收藏文章"
                : searchKeyword
                ? "未找到相关文章"
                : "暂无资讯"
            }
            emptyHint={
              tab === "saved"
                ? "浏览资讯并收藏感兴趣的文章"
                : searchKeyword
                ? "尝试使用不同的关键词"
                : "点击下方按钮刷新获取最新资讯"
            }
            emptyActionText={
              tab === "saved"
                ? "去浏览"
                : searchKeyword
                ? "清空搜索"
                : "REFRESH"
            }
            onEmptyAction={
              tab === "saved"
                ? () => setTab("search")
                : searchKeyword
                ? () => {
                    setSearchKeyword("");
                    void loadArticles(1, false);
                  }
                : runCrawler
            }
          />

          {/* 页码导航 */}
          {!searchKeyword && listItems.length > 0 && calculatedTotalPages > 1 && (
            <div className="pagination">
              <button
                type="button"
                className="pagination-btn"
                onClick={() => goToPage(currentPage - 1)}
                disabled={currentPage <= 1}
              >
                ◀ 上一页
              </button>

              <div className="pagination-info">
                <span className="pagination-current">{currentPage}</span>
                <span className="pagination-divider">/</span>
                <span className="pagination-total">{calculatedTotalPages}</span>
              </div>

              <button
                type="button"
                className="pagination-btn"
                onClick={() => goToPage(currentPage + 1)}
                disabled={currentPage >= calculatedTotalPages}
              >
                下一页 ▶
              </button>
            </div>
          )}

          </>
        )}

        {tab === "settings" && (
          <section className="settings-grid">
            <label>
              主题
              <select
                value={settings.theme}
                onChange={(event) =>
                  setSettings((prev) => ({ ...prev, theme: event.target.value as Settings["theme"] }))
                }
              >
                <option value="auto">🌓 自动</option>
                <option value="dark">🌙 深色</option>
                <option value="light">☀️ 浅色</option>
              </select>
            </label>
            <button type="button" className="btn-primary" onClick={saveSettings} disabled={loading}>
              {loading ? "保存中..." : "💾 保存设置"}
            </button>

            <div style={{ marginTop: "20px", borderTop: "1px solid var(--border)", paddingTop: "20px" }}>
              <h3 style={{ margin: "0 0 12px 0", color: "var(--accent-2)" }}>🔄 批量更新摘要</h3>
              <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "12px" }}>
                使用 AI 重新生成所有模板摘要（显示"这篇英文资讯围绕..."的文章）
              </p>

              {!updateProgress.isRunning ? (
                <button
                  type="button"
                  className="btn-primary"
                  onClick={async () => {
                    if (!confirm("确定要批量更新所有模板摘要吗？这可能需要较长时间。")) return;
                    setUpdateProgress({
                      isRunning: true,
                      total: null,
                      current: 0,
                      updated: 0,
                      currentTitle: null,
                      error: null,
                    });
                    try {
                      await api.regenerateSummaries();
                    } catch (error) {
                      setUpdateProgress((prev) => ({
                        ...prev,
                        isRunning: false,
                        error: String(error),
                      }));
                    }
                  }}
                  disabled={loading}
                >
                  🤖 批量更新摘要
                </button>
              ) : (
                <SummaryUpdateProgress
                  status={updateProgress}
                  onClose={() => {
                    // Allow closing settings while update runs
                    setTab("search");
                  }}
                />
              )}
            </div>

            <div style={{ marginTop: "20px", borderTop: "1px solid var(--border)", paddingTop: "20px" }}>
              <h3 style={{ margin: "0 0 12px 0", color: "var(--accent-2)" }}>🤖 AI 摘要测试</h3>
              <textarea
                value={summaryInput}
                onChange={(event) => setSummaryInput(event.target.value)}
                placeholder="粘贴文章内容进行 AI 摘要测试..."
              />
              <button
                type="button"
                className="btn-primary"
                onClick={summarize}
                disabled={loading || !summaryInput.trim()}
              >
                {loading ? "生成中..." : "✨ 生成摘要"}
              </button>
              {summaryOutput && <pre>{summaryOutput}</pre>}
            </div>
          </section>
        )}
      </section>

      <footer className="panel statusbar">
        <span className={`state-dot ${uiState}`}>
          {uiState === "loading" && "⏳"}
          {uiState === "success" && "✅"}
          {uiState === "error" && "❌"}
          {uiState === "idle" && "💤"}
        </span>
        <span>{statusMessage}</span>
        <span>📅 {lastUpdated}</span>
        <span>📊 共 {totalArticles} 篇</span>
        {tab !== "search" && (
          <span>📄 当前 {listItems.length} 篇 / {articlesPerPage}/页</span>
        )}
        <span>🆕 今日 {todayCount} 篇</span>
      </footer>

      {/* Toast 消息 */}
      {toast && (
        <div className={`toast ${toast.type}`}>
          {toast.type === "success" && "✅ "}
          {toast.type === "error" && "❌ "}
          {toast.type === "warning" && "⚠️ "}
          {toast.message}
        </div>
      )}
    </main>
  );
}
