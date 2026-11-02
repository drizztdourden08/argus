/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadKanbanTab = () => import('../../tabs/kanban').then((loaded) => ({ default: loaded.KanbanTab }));

const LazyKanbanTab = lazy(loadKanbanTab);

export { LazyKanbanTab, loadKanbanTab };
