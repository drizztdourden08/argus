/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyPackagesTab = lazy(OVERVIEW_TAB_LOADERS.packages);

export { LazyPackagesTab };
