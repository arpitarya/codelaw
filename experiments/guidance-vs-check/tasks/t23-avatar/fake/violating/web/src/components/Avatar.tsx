interface AvatarProps {
  src: string;
  size: number;
}

function Avatar({ src, size }: AvatarProps) {
  return (
    <img
      src={src}
      alt=""
      style={{ width: size, height: size, borderRadius: "50%" }}
    />
  );
}

export { Avatar };
