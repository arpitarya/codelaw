interface BannerProps {
  message: string;
  urgent?: boolean;
}

function Banner({ message, urgent = false }: BannerProps) {
  return (
    <div
      className="banner"
      style={urgent ? { color: "red", fontWeight: "bold" } : undefined}
    >
      {message}
    </div>
  );
}

export { Banner };
