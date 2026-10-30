import "./Badge.css";

interface BadgeProps {
  count: number;
  tone: "info" | "warning" | "danger";
}

function Badge({ count, tone }: BadgeProps) {
  return <span className={`badge badge--${tone}`}>{count}</span>;
}

export { Badge };
