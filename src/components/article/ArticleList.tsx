import { ArticleCard } from "./ArticleCard";
import { EmptyState } from "../common/EmptyState";
import type { Article } from "../../types";

interface ArticleListProps {
  items: Article[];
  onToggleBookmark: (id: string, value: boolean) => Promise<void>;
  onRefresh?: () => void;
  emptyMessage?: string;
  emptyHint?: string;
  emptyActionText?: string;
  onEmptyAction?: () => void;
}

export function ArticleList({
  items,
  onToggleBookmark,
  onRefresh,
  emptyMessage = "暂无内容",
  emptyHint = "点击下方按钮刷新获取最新资讯",
  emptyActionText = "刷新资讯",
  onEmptyAction,
}: ArticleListProps): JSX.Element {
  if (items.length === 0) {
    return (
      <EmptyState
        icon="📭"
        message={emptyMessage}
        hint={emptyHint}
        actionText={emptyActionText}
        onAction={onEmptyAction || onRefresh}
      />
    );
  }

  return (
    <ul className="feed-list">
      {items.map((item) => (
        <ArticleCard
          key={item.id}
          article={item}
          onToggleBookmark={onToggleBookmark}
        />
      ))}
    </ul>
  );
}
