/* @layer renderer-app @kind types */
interface BoardMeta {
  project: string;
  name: string;
  updatedAt: string;
  palette?: string;
  avatar?: string | null;
  icon?: string | null;
  owner?: { name?: string; initials?: string; avatar?: string | null };
}

interface ThreadFlags {
  needsHuman: boolean;
  blocked: boolean;
}

interface Thread {
  id: string;
  name: string;
  role: string;
  status: string;
  activity?: string;
  note?: string | null;
  flags?: ThreadFlags;
  rank?: number;
  mergedInto?: string;
  deletedAt?: string;
}

export type { BoardMeta, Thread };
