const TONES = { info: "#2563eb", warning: "#d97706", danger: "#dc2626" };

interface BadgeProps {
  count: number;
  tone: "info" | "warning" | "danger";
}

function Badge({ count, tone }: BadgeProps) {
  return (
    <span className="badge" style={{ backgroundColor: TONES[tone] }}>
      {count}
    </span>
  );
}

export { Badge };
