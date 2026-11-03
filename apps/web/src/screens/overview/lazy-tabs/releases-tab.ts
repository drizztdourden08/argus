/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyReleasesTab = lazy(OVERVIEW_TAB_LOADERS.releases);

export { LazyReleasesTab };
