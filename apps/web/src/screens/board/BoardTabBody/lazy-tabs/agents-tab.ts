/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyAgentsTab = lazy(BOARD_TAB_LOADERS.agents);

export { LazyAgentsTab };
