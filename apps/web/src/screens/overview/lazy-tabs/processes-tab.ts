/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyProcessesTab = lazy(OVERVIEW_TAB_LOADERS.processes);

export { LazyProcessesTab };
