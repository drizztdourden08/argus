/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyActivityTab = lazy(BOARD_TAB_LOADERS.activity);

export { LazyActivityTab };
