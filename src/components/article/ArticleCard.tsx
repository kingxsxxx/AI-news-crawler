import { CategoryBadge } from "../common/CategoryBadge";
import { HeatIndicator } from "../common/HeatIndicator";
import { api } from "../../lib/api";

interface ArticleCardProps {
  article: {
    id: string;
    title: string;
    summary: string;
    url: string;
    source: string;
    category: string;
    published_at?: string;
    fetched_at: string;
    heat_score: number;
    image_url?: string;
    is_bookmarked: boolean;
  };
  onToggleBookmark: (id: string, value: boolean) => Promise<void>;
}

export function ArticleCard({ article, onToggleBookmark }: ArticleCardProps): JSX.Element {
  const fallbackImage = (): string =>
    `https://picsum.photos/seed/${encodeURIComponent(
      `${article.source}-${article.title}`.toLowerCase()
    )}/360/220`;

  const formatDate = (value?: string): string => {
    if (!value) return "Unknown date";
    const dt = new Date(value);
    if (Number.isNaN(dt.getTime())) return value;
    const now = new Date();
    const diffMs = now.getTime() - dt.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffHours / 24);

    if (diffHours < 1) return "刚刚";
    if (diffHours < 24) return `${diffHours}小时前`;
    if (diffDays < 7) return `${diffDays}天前`;
    return dt.toLocaleDateString("zh-CN");
  };

  const handleTitleClick = (event: React.MouseEvent): void => {
    event.preventDefault();
    void api.openExternal(article.url);
  };

  const handleBookmarkClick = (): void => {
    void onToggleBookmark(article.id, !article.is_bookmarked);
  };

  return (
    <li className="news-card">
      <img
        src={article.image_url || fallbackImage()}
        alt={article.title}
        className="news-thumb"
        loading="lazy"
        onError={(event) => {
          const target = event.currentTarget;
          if (target.src !== fallbackImage()) {
            target.src = fallbackImage();
          }
        }}
      />
      <div className="news-body">
        <div className="article-header">
          <div className="article-badges">
            <CategoryBadge category={article.category} />
            <HeatIndicator score={article.heat_score} />
          </div>
          <div className="article-actions">
            <button
              type="button"
              className="btn-ghost save-btn"
              onClick={handleBookmarkClick}
              title={article.is_bookmarked ? "取消收藏" : "收藏"}
            >
              {article.is_bookmarked ? "★" : "☆"}
            </button>
          </div>
        </div>
        <a
          href={article.url}
          className="news-title"
          onClick={handleTitleClick}
        >
          {article.title}
        </a>
        <div className="news-meta">
          <strong>{article.source}</strong>
          <span>{formatDate(article.published_at || article.fetched_at)}</span>
        </div>
        <p className="news-summary">{article.summary || "暂无中文简介。"}</p>
      </div>
    </li>
  );
}
