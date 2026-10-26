/* @layer renderer-app @kind types */
interface Wake {
  subject?: string | null;
  thread?: string | null;
  status?: string;
  listeners?: number;
  at?: string;
  into?: string | null;
}

export type { Wake };
