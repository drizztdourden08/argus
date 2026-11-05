/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { PAGE_LOADERS } from './loaders.constants';

const LazyOverviewMode = lazy(PAGE_LOADERS.overviewMode);

export { LazyOverviewMode };
