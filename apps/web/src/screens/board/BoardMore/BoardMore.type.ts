/* @layer renderer-app @kind types */
import type { TagLook } from '@drizztdourden08/tessera';
import type { BrandApp } from '@drizztdourden08/tessera/brand';
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

type MoreDialog = 'locks' | 'legend' | 'avatar' | null;

interface MoreDialogsProps {
  boardKey: string;
  dialog: MoreDialog;
  locks: MoreCounts['locks'];
  held: number;
  onClose: () => void;
}

interface AvatarDialogProps {
  boardKey: string;
  onClose: () => void;
}

interface AvatarChoice {
  setting: string | null;
  busy: boolean;
  problem: string | null;
  pickBrand: (app: BrandApp) => Promise<void>;
  upload: (files: File[]) => Promise<void>;
  reset: () => Promise<void>;
}

interface LegendEntry {
  label: string;
  look: TagLook;
  text: string;
}

export type { AvatarChoice, AvatarDialogProps, BoardMoreProps, BoardMoreState, LegendEntry, Lock, MoreCounts, MoreDialog, MoreDialogsProps };
