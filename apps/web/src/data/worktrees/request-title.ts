/* @layer renderer-app @kind logic */
import type { WorktreeRequest } from './worktrees.type';
import { requestWhat } from './request-what';

const requestTitle = (request: WorktreeRequest): string => `${request.by} asked to remove ${requestWhat(request)}`;

export { requestTitle };
