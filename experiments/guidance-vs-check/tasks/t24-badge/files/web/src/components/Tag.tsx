interface TagProps {
  label: string;
  color: string;
}

function Tag({ label, color }: TagProps) {
  return (
    <span className="tag" style={{ backgroundColor: color }}>
      {label}
    </span>
  );
}

export { Tag };
