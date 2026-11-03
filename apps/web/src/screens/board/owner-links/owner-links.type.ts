/* @layer renderer-app @kind types */
import type { PostResult } from '../../../data/api';

type OwnerAction = 'worktree-link' | 'worktree-unlink' | 'worktree-keep' | 'launcher-link' | 'launcher-unlink' | 'launcher-retire';

interface OwnerAsk {
  action: OwnerAction;
  id: string;
  label: string;
}

interface OwnerAskState {
  ask: OwnerAsk | null;
  open: (ask: OwnerAsk) => void;
  close: () => void;
}

interface OwnerSend {
  busy: boolean;
  problem: string | null;
  send: (post: () => Promise<PostResult>, done: string) => Promise<void>;
}

interface OwnerDialogProps {
  boardKey: string;
  ask: OwnerAsk;
  onClose: () => void;
}

export type { OwnerAction, OwnerAsk, OwnerAskState, OwnerDialogProps, OwnerSend };
