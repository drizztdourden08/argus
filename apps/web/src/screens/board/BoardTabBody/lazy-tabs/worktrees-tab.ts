/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyWorktreesTab = lazy(() => import('../../tabs/worktrees').then((loaded) => ({ default: loaded.WorktreesTab })));

export { LazyWorktreesTab };
