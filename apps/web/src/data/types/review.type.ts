/* @layer renderer-app @kind types */
type ReviewStatus =
  | 'same' | 'missing' | 'will_create' | 'only_here' | 'folder_differs'
  | 'archived_here_only' | 'active_here_only' | 'not_subthread' | 'subthread' | 'empty';

interface ReviewCell {
  present?: boolean;
  folder?: string | null;
  archived?: boolean;
  lastActivityAt?: number | string | null;
  lastFocusedAt?: number | string | null;
  parentId?: string | null;
  status?: ReviewStatus;
}

type ReviewDiffKind = 'chat_missing' | 'history' | 'archive' | 'membership' | 'thread' | 'folder_missing';

interface ReviewChoice {
  id: string;
  label: string;
}

interface ReviewDiff {
  kind: ReviewDiffKind;
  choices: ReviewChoice[];
  default: string;
}

interface ReviewRow {
  id: string;
  kind: 'folder' | 'chat';
  depth: number;
  parentId?: string | null;
  folderId?: string | null;
  title: string;
  cells: Record<string, ReviewCell | undefined>;
  diff: ReviewDiff | null;
}

interface ReviewAccount {
  key: string;
  label?: string;
  color?: string;
  scope?: string;
}

interface ReviewDoc {
  scannedAt?: string | null;
  accounts: ReviewAccount[];
  rows: ReviewRow[];
  counts?: { chats?: Record<string, number>; differences?: number; byKind?: Record<string, number> };
}

type ReviewEdit =
  | { op: 'move_chat'; account: string; sessionId: string; folder: string | null }
  | { op: 'archive' | 'unarchive' | 'delete_chat'; account: string; sessionId: string }
  | { op: 'rename_folder'; account: string; folder: string; name: string }
  | { op: 'delete_folder'; account: string; folder: string }
  | { op: 'create_folder'; account: string; name: string };

interface CleanupCandidate {
  account: string;
  sessionId: string;
  title?: string | null;
  folder?: string | null;
  lastActivityAt?: number | string | null;
}

interface CleanupDoc {
  days: number;
  candidates: CleanupCandidate[];
}

export type {
  CleanupCandidate, CleanupDoc, ReviewAccount, ReviewCell, ReviewDiff, ReviewDoc, ReviewEdit, ReviewRow, ReviewStatus,
};
