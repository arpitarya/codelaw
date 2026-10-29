import { flagsmith } from "./sdk/clients";

// codelaw-flag fs-js expires 2026-11-10 owner @web
export const search = flagsmith.hasFeature("fs-js");
