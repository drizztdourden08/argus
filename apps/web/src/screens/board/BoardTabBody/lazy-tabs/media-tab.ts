/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { BOARD_TAB_LOADERS } from './loaders.constants';

const LazyMediaTab = lazy(BOARD_TAB_LOADERS.media);

export { LazyMediaTab };
