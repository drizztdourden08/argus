/* @layer renderer-app @kind types */
interface Wake {
  reason?: string;
  text?: string;
  subject?: string | null;
  thread?: string | null;
  status?: string;
  listeners?: number;
  at?: string;
  into?: string | null;
}

export type { Wake };
