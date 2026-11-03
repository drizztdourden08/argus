/* @layer renderer-app @kind logic */
import type { WorktreeRequest } from '../../../../../data/worktrees';
import type { RowChip } from '../../../../../shell';
import { REQUEST_LOOK, REQUEST_TIP, REQUEST_WAITING } from './model.constants';

const countsChip = (request: WorktreeRequest): RowChip[] => {
  if (REQUEST_WAITING.includes(request.status)) return [];
  const refused = request.refused ? `, ${request.refused} refused` : '';
  const skipped = request.skipped ? `, ${request.skipped} skipped` : '';
  return [{ id: 'counts', text: `${request.removed} removed${refused}${skipped}` }];
};

const requestChips = (request: WorktreeRequest): RowChip[] => [
  { id: 'status', text: request.status, title: REQUEST_TIP[request.status] ?? request.status, look: REQUEST_LOOK[request.status] },
  ...countsChip(request),
];

export { requestChips };
