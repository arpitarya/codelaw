interface ProgressBarProps {
  percent: number;
}

function ProgressBar({ percent }: ProgressBarProps) {
  const width = Math.min(100, Math.max(0, percent));
  return (
    <div className="progress">
      <div className="progress__fill" style={{ width: `${width}%` }} />
    </div>
  );
}

export { ProgressBar };
