/* @layer renderer-app @kind logic */
import type { PostResult } from '../../../../data/api';
import { orText } from '../../format';
import type { SendOutcome } from './model.type';

const sendOutcome = (result: PostResult, name: string): SendOutcome => {
  if (result.status === 'refused' || result.status === 'conflict') {
    return { ok: false, message: `Refused: ${orText(result.outcome, result.status)}.` };
  }
  if (result.status === 'waiting for a board') return { ok: true, message: `${result.id} is kept for ${name} until its board exists.` };
  return { ok: true, message: `${result.id} sent down to ${name}.` };
};

export { sendOutcome };
