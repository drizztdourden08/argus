/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyWorktreesTab = lazy(BOARD_TAB_LOADERS.worktrees);

export { LazyWorktreesTab };
