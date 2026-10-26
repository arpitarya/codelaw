import { CodelawPlugin } from "@codelaw/webpack-plugin";

export default {
  mode: "development",
  entry: "./src/main.js",
  devServer: { host: "127.0.0.1", client: { overlay: { errors: true, warnings: true } } },
  plugins: [new CodelawPlugin()],
};
