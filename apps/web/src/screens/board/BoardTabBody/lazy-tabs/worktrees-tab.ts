/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadWorktreesTab = () => import('../../tabs/worktrees').then((loaded) => ({ default: loaded.WorktreesTab }));

const LazyWorktreesTab = lazy(loadWorktreesTab);

export { LazyWorktreesTab, loadWorktreesTab };
