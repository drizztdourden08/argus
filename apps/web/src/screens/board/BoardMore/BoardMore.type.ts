/* @layer renderer-app @kind types */
import type { PurgeRequest } from '../tabs/kanban';

interface BoardMoreProps {
  trashCount: number;
  onTrash: () => void;
  onPurge: () => void;
}

interface BoardMoreState {
  trashOpen: boolean;
  purge: PurgeRequest | null;
  openTrash: () => void;
  closeTrash: () => void;
  openPurge: (request: PurgeRequest) => void;
  closePurge: () => void;
}

export type { BoardMoreProps, BoardMoreState };
