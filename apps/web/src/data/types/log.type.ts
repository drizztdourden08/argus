/* @layer renderer-app @kind types */
interface LogLine {
  seq: number;
  ts: string;
  thread: string;
  item?: string | null;
  text: string;
  subject?: string;
}

interface LogDoc {
  subject: string;
  entries: LogLine[];
}

export type { LogDoc };
