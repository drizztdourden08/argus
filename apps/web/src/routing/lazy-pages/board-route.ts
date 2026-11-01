/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyBoardRoute = lazy(() => import('../../screens/board/BoardRoute').then((loaded) => ({ default: loaded.BoardRoute })));

export { LazyBoardRoute };
