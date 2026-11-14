/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyOverlordTab = lazy(OVERVIEW_TAB_LOADERS.overlord);

export { LazyOverlordTab };
