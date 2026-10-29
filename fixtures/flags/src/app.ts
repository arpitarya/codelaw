import { enabled } from "./lib/feature";

export function render(): string {
  if (enabled("wrapped")) {
    return "new";
  } else {
    return "old";
  }
}
