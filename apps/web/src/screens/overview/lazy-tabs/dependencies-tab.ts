/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyDependenciesTab = lazy(OVERVIEW_TAB_LOADERS.dependencies);

export { LazyDependenciesTab };
