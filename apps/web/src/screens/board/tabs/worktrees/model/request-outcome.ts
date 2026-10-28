/* @layer renderer-app @kind logic */
import type { WorktreeRequest } from '../../../../../data/worktrees';
import { plainText } from '../../../plain-text';
import { REQUEST_WAITING } from './model.constants';

const countsText = (request: WorktreeRequest): string => {
  if (REQUEST_WAITING.includes(request.status)) return '';
  const refused = request.refused ? `, ${request.refused} refused` : '';
  const skipped = request.skipped ? `, ${request.skipped} skipped` : '';
  return ` · ${request.removed} removed${refused}${skipped}`;
};

const requestOutcome = (request: WorktreeRequest): string => {
  const note = request.note ? ` · ${plainText(request.note)}` : '';
  return `${countsText(request)}${note}`;
};

export { requestOutcome };
