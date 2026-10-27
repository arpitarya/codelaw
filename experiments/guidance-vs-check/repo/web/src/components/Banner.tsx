interface BannerProps {
  message: string;
}

function Banner({ message }: BannerProps) {
  return <div className="banner">{message}</div>;
}

export { Banner };
