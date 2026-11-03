/* @layer renderer-app @kind logic */
import type { WorktreeRequest } from '../../../../../data/worktrees';
import { requestWhat } from './request-what';

const requestTitle = (request: WorktreeRequest): string => `${request.by} asked to remove ${requestWhat(request)}`;

export { requestTitle };
