/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyRegistryTab = lazy(OVERVIEW_TAB_LOADERS.registry);

export { LazyRegistryTab };
