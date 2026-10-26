/* @layer renderer-app @kind types */
interface Timed {
  at?: string;
}

interface Unread<T extends Timed> {
  unread: T[];
  freshSince: number;
}

export type { Timed, Unread };
