/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import type { Verdict } from '../model';

const announce = (verdict: Verdict, id: string, undo?: () => void): void => {
  toast({ id, message: verdict.message, variant: verdict.ok ? 'info' : 'danger', undo: verdict.ok ? undo : undefined });
};

export { announce };
