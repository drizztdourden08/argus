/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { PAGE_LOADERS } from './loaders.constants';

const LazyBoardRoute = lazy(PAGE_LOADERS.boardRoute);

export { LazyBoardRoute };
