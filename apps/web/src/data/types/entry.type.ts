/* @layer renderer-app @kind types */
interface Entry {
  id: string;
  kind: string;
  author: string;
  text: string;
  status?: string;
  thread?: string | null;
  item?: string | null;
  replyTo?: string | null;
  context?: string;
  options?: string[];
  needs?: string;
  level?: string;
  createdAt: string;
  appliedAt?: string;
  archivedAt?: string;
  deletedAt?: string;
}

export type { Entry };
