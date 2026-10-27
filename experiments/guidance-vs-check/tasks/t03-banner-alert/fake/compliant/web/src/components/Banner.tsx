interface BannerProps {
  message: string;
  urgent?: boolean;
}

function Banner({ message, urgent = false }: BannerProps) {
  return (
    <div className={urgent ? "banner banner--urgent" : "banner"}>
      {message}
    </div>
  );
}

export { Banner };
