/* @layer renderer-app @kind types */
interface Slot {
  set: (node: HTMLElement | null) => void;
  subscribe: (listener: () => void) => () => void;
  get: () => HTMLElement | null;
}

export type { Slot };
