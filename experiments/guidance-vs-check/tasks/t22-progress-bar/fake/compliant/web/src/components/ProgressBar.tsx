interface ProgressBarProps {
  percent: number;
}

function ProgressBar({ percent }: ProgressBarProps) {
  const value = Math.min(100, Math.max(0, percent));
  return <progress className="progress" value={value} max={100} />;
}

export { ProgressBar };
