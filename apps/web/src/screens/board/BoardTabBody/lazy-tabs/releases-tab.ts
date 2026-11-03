/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyReleasesTab = lazy(BOARD_TAB_LOADERS.releases);

export { LazyReleasesTab };
