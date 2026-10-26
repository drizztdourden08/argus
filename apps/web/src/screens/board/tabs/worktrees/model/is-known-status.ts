/* @layer renderer-app @kind logic */
import { WORKTREE_STATUSES } from './model.constants';
import type { KnownStatus } from './model.type';

const isKnownStatus = (status: string): status is KnownStatus => status in WORKTREE_STATUSES;

export { isKnownStatus };
