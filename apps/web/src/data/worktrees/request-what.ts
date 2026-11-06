/* @layer renderer-app @kind logic */
import type { WorktreeRequest } from './worktrees.type';

const countText = (request: WorktreeRequest): string => `${request.total} worktree${request.total === 1 ? '' : 's'}`;

const requestWhat = (request: WorktreeRequest): string => {
  const kinds: Record<string, string> = {
    finished: 'every finished worktree',
    leftovers: 'every old leftover folder',
    detached: `its ${countText(request)} nothing was attached to`,
    approved: `its ${countText(request)} approved for removal, full backup first`,
  };
  const what = kinds[request.mode] ?? countText(request);
  return request.branch ? `${what} and merged branches` : what;
};

export { requestWhat };
