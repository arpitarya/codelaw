import { ldClient } from "./sdk/clients";

export const banner = ldClient.variation("ld-js", { key: "user" }, false);
