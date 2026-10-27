function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise(async (resolve, reject) => {
    const img = new Image();
    img.src = src;
    try {
      await img.decode();
      resolve(img);
    } catch (err) {
      reject(err);
    }
  });
}

export { loadImage };
