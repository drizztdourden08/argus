/* @layer renderer-app @kind logic */
import type { WorktreeRequest } from '../../../../../data/worktrees';
import { plainText } from '../../../plain-text';

const requestNote = (request: WorktreeRequest): string =>
  [request.note, ...(request.sentences ?? [])]
    .filter((part): part is string => Boolean(part))
    .map(plainText)
    .join(' · ');

export { requestNote };
