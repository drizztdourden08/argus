/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { PAGE_LOADERS } from './loaders.constants';

const LazyBoardMode = lazy(PAGE_LOADERS.boardMode);

export { LazyBoardMode };
