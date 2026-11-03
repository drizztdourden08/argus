/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyCacheTab = lazy(OVERVIEW_TAB_LOADERS.cache);

export { LazyCacheTab };
