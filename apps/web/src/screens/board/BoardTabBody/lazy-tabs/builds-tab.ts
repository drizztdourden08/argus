/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyBuildsTab = lazy(BOARD_TAB_LOADERS.builds);

export { LazyBuildsTab };
