import { ldClient } from "../sdk/clients";

/** The team's wrapper over LaunchDarkly. */
export function isOn(name: string): boolean {
  return ldClient.variation(name, { key: "user" }, false);
}
