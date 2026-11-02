/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyActivityTab = lazy(OVERVIEW_TAB_LOADERS.activity);

export { LazyActivityTab };
