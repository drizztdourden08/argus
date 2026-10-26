/* @layer renderer-app @kind types */
interface BoardAction {
  id: string;
  text: string;
  status: string;
  thread: string;
  item?: string | null;
  createdAt: string;
  archivedAt?: string;
  deletedAt?: string;
}

export type { BoardAction };
