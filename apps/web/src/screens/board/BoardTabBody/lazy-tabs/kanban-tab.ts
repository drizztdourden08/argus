/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyKanbanTab = lazy(() => import('../../tabs/kanban').then((loaded) => ({ default: loaded.KanbanTab })));

export { LazyKanbanTab };
