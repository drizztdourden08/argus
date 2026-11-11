/* @layer renderer-app @kind types */
interface SwitchTarget {
  key: string | null;
  open: (key: string) => void;
  close: () => void;
}

export type { SwitchTarget };
