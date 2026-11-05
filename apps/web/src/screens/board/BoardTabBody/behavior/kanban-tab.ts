/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyKanbanTab = lazy(BOARD_TAB_LOADERS.kanban);

export { LazyKanbanTab };
