/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyTestsTab = lazy(BOARD_TAB_LOADERS.tests);

export { LazyTestsTab };
