/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadBoardRoute = () => import('../../screens/board/BoardRoute').then((loaded) => ({ default: loaded.BoardRoute }));

const LazyBoardRoute = lazy(loadBoardRoute);

export { LazyBoardRoute, loadBoardRoute };
