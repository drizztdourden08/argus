/* @layer renderer-app @kind types */
import type { TagLook } from '@drizztdourden08/tessera';
import type { PurgeRequest } from '../tabs/kanban';

interface BoardMoreProps {
  boardKey: string;
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

interface Lock {
  resource?: string;
  holder?: string | null;
  since?: string | null;
  queue?: string[];
  description?: string | null;
}

interface MoreCounts {
  held: number;
  hidden: number;
  locks: readonly (Lock & { id: string })[];
}

type MoreDialog = 'locks' | 'legend' | null;

interface MoreDialogsProps {
  dialog: MoreDialog;
  locks: MoreCounts['locks'];
  held: number;
  onClose: () => void;
}

interface LegendEntry {
  label: string;
  look: TagLook;
  text: string;
}

export type { BoardMoreProps, BoardMoreState, LegendEntry, Lock, MoreCounts, MoreDialog, MoreDialogsProps };
