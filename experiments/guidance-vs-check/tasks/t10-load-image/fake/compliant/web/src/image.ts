function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.src = src;
    img.decode().then(() => resolve(img), reject);
  });
}

export { loadImage };
