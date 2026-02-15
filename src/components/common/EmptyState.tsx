interface EmptyStateProps {
  icon?: string;
  message: string;
  hint?: string;
  actionText?: string;
  onAction?: () => void;
}

export function EmptyState({
  icon = "📭",
  message,
  hint,
  actionText,
  onAction,
}: EmptyStateProps): JSX.Element {
  return (
    <div className="empty-state">
      <div className="empty-icon">{icon}</div>
      <div className="empty-message">{message}</div>
      {hint && <div className="empty-hint">{hint}</div>}
      {actionText && onAction && (
        <button type="button" className="empty-action" onClick={onAction}>
          {actionText}
        </button>
      )}
    </div>
  );
}
