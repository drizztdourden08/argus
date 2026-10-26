/* @layer renderer-app @kind logic */
import type { PostResult } from '../../../../../data/api';
import { CONFLICT_TEXT, REFUSED_TEXT } from './model.constants';
import type { Verdict } from './model.type';

const postVerdict = (result: PostResult, done: string): Verdict => {
  if (result.status === 'conflict') return { ok: false, message: `Conflict: ${result.outcome ?? CONFLICT_TEXT}. Nothing of yours was applied.` };
  if (result.status === 'refused') return { ok: false, message: `Refused: ${result.outcome ?? REFUSED_TEXT}.` };
  return { ok: true, message: done };
};

export { postVerdict };
