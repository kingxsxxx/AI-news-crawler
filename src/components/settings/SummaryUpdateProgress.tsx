import type { SummaryUpdateStatus } from "../../types";

interface SummaryUpdateProgressProps {
  status: SummaryUpdateStatus;
  onClose?: () => void;
}

export function SummaryUpdateProgress({ status, onClose }: SummaryUpdateProgressProps): JSX.Element {
  const percentage = status.total ? Math.round((status.current / status.total) * 100) : 0;

  return (
    <div className="update-progress-container">
      <div className="update-progress-header">
        <h3>批量更新摘要</h3>
        {onClose && (
          <button type="button" className="btn-ghost" onClick={onClose} disabled={status.isRunning}>
            ✕
          </button>
        )}
      </div>

      <div className="update-progress-stats">
        <div className="stat-item">
          <span className="stat-label">进度</span>
          <span className="stat-value">
            {status.current} / {status.total || '-'}
          </span>
        </div>
        <div className="stat-item">
          <span className="stat-label">已更新</span>
          <span className="stat-value">{status.updated} 篇</span>
        </div>
        <div className="stat-item">
          <span className="stat-label">完成度</span>
          <span className="stat-value">{percentage}%</span>
        </div>
      </div>

      <div className="progress-bar-container">
        <div className="progress-bar-fill" style={{ width: `${percentage}%` }} />
      </div>

      {status.currentTitle && (
        <div className="current-article">
          <span className="current-label">正在处理:</span>
          <span className="current-title">{status.currentTitle}</span>
        </div>
      )}

      {status.error && (
        <div className="error-message">
          <span>错误: {status.error}</span>
        </div>
      )}

      {status.isRunning && (
        <div className="loading-indicator">
          <span className="spinner"></span>
          <span>更新中...</span>
        </div>
      )}
    </div>
  );
}
