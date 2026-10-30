import "./Avatar.css";

interface AvatarProps {
  src: string;
  size: number;
}

function Avatar({ src, size }: AvatarProps) {
  return <img className="avatar" src={src} alt="" width={size} height={size} />;
}

export { Avatar };
