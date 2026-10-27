/* @layer renderer-app @kind types */
interface Criterion {
  n: number;
  text: string;
  done: boolean;
}

interface WorkItem {
  id: string;
  title: string;
  status: string;
  thread: string;
  subject?: string;
  description?: string;
  ac?: Criterion[];
  rank?: number;
  updatedAt?: string;
  archivedAt?: string;
  deletedAt?: string;
}

export type { WorkItem };
