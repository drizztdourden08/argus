/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyChangesTab = lazy(BOARD_TAB_LOADERS.changes);

export { LazyChangesTab };
