/* @layer renderer-app @kind types */
interface ControlSet {
  start: boolean;
  restart: boolean;
  stop: boolean;
}

interface SpendLook {
  ratio: number;
  tone: 'success' | 'warning' | 'danger';
}

export type { ControlSet, SpendLook };
