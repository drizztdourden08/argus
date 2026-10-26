/* @layer renderer-app @kind logic */
import type { PostResult } from '../../../../../data/api';

const entryProblem = (result: PostResult): string | null => {
  const why = result.outcome;
  if (result.status === 'conflict') return `Conflict: ${why ?? 'a thread changed it after you looked'}. Nothing of yours was applied.`;
  if (result.status === 'refused') return `Refused: ${why ?? 'the board did not accept it'}.`;
  return null;
};

export { entryProblem };
