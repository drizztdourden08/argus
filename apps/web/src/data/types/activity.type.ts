/* @layer renderer-app @kind types */
type ActivityLevel = 'info' | 'warn' | 'error';

interface ActivityDoc {
  n?: number;
  at?: string;
  kind?: string;
  level?: string;
  text?: string;
  project?: string | null;
  thread?: string | null;
}

export type { ActivityDoc, ActivityLevel };
