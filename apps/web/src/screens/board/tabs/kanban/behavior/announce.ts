/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import type { OpVerdict } from '../../../../../data/kanban';
import { UNDO_TOAST_MS } from '../kanban.constants';

const announce = (verdict: OpVerdict, id: string, undo?: () => void): void => {
  const undoable = verdict.ok && undo !== undefined;
  toast({
    id,
    message: verdict.message,
    variant: verdict.ok ? 'success' : 'danger',
    undo: undoable ? undo : undefined,
    duration: undoable ? UNDO_TOAST_MS : undefined,
  });
};

export { announce };
