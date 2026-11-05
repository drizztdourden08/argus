/* @layer renderer-app @kind types */
import type { WithId } from '../react';
import type { Card } from '../types';

interface BoardDoc {
  key: string;
  rev?: number;
  goal?: string | null;
  note?: string | null;
  name?: string;
  icon?: string;
  createdAt?: string;
  createdBy?: string;
  updatedAt?: string;
  by?: string;
  archivedAt?: string | null;
  archivedBy?: string | null;
}

interface BoardItem {
  doc: WithId<BoardDoc>;
  card: Card | null;
}

interface BoardItems {
  ready: boolean;
  items: BoardItem[];
}

type BoardEditField = 'goal' | 'note';

type BoardOp =
  | { kind: 'edit'; id: string; field: BoardEditField; value: string; prev: string | null; baseRev?: number }
  | { kind: 'archive' | 'unarchive'; id: string };

export type { BoardDoc, BoardItem, BoardItems, BoardOp };
