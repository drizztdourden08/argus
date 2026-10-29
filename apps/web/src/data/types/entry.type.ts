/* @layer renderer-app @kind types */
interface EntryImage {
  n: number;
  url: string;
  caption: string;
  option?: number | null;
}

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
  images?: EntryImage[];
  needs?: string;
  level?: string;
  createdAt: string;
  appliedAt?: string;
  archivedAt?: string;
  deletedAt?: string;
}

export type { Entry, EntryImage };
