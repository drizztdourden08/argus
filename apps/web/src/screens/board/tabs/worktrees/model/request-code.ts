/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../../data/registry';
import type { WorktreeRequest } from '../../../../../data/worktrees';
import { shownCode } from '../../../../../shell/records';
import { REQUEST_KIND } from '../worktrees.constants';

const requestCode = (request: WorktreeRequest): string => {
  const kind = kindOf(REQUEST_KIND);
  return kind ? shownCode(kind, request.id) : request.id;
};

export { requestCode };
