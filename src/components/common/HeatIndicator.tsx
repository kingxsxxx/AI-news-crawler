interface HeatIndicatorProps {
  score: number;
  showScore?: boolean;
}

export function HeatIndicator({ score, showScore = true }: HeatIndicatorProps): JSX.Element {
  const getHeatLevel = (value: number): "high" | "medium" | "low" => {
    if (value >= 60) return "high";
    if (value >= 30) return "medium";
    return "low";
  };

  const heatLevel = getHeatLevel(score);

  return (
    <div className={`heat-indicator ${heatLevel}`}>
      {showScore && <span>{score.toFixed(0)}</span>}
      <div className="heat-bar" title={`热度: ${score.toFixed(1)}`} />
    </div>
  );
}
