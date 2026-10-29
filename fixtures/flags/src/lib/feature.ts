import { isOn } from "./flags";

/** A wrapper of the wrapper. */
export const enabled = (flag: string): boolean => isOn(flag);
