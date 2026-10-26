/* @layer renderer-app @kind logic */
import type { PostResult } from '../../../../../data/api';

const postVerdict = (result: PostResult): string | null => {
  if (result.status === 'conflict') return `Conflict: ${result.outcome ?? 'a thread changed it after you looked'}. Nothing of yours was applied.`;
  if (result.status === 'refused') return `Refused: ${result.outcome ?? 'the board did not accept it'}.`;
  return null;
};

export { postVerdict };
