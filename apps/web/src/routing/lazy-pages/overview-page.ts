/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { PAGE_LOADERS } from './loaders.constants';

const LazyOverviewPage = lazy(PAGE_LOADERS.overviewPage);

export { LazyOverviewPage };
