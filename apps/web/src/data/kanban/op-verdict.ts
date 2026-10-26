/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { CONFLICT_REASON, REFUSED_REASON } from './kanban.constants';
import type { OpVerdict } from './kanban.type';

const opVerdict = (result: PostResult, done: string): OpVerdict => {
  if (result.status === 'conflict') return { ok: false, message: `Conflict: ${result.outcome ?? CONFLICT_REASON}. Nothing of yours was applied (see the card).` };
  if (result.status === 'refused') return { ok: false, message: `Refused: ${result.outcome ?? REFUSED_REASON}.` };
  return { ok: true, message: done };
};

export { opVerdict };
