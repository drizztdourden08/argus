/* @layer renderer-app @kind component */
import { Suspense } from 'react';
import { Route, Routes } from 'react-router';
import { NotFoundPage } from '../screens/not-found/NotFoundPage';
import { LoadingFallback, PageFrame } from '../shell';
import { LazyBoardRoute, LazyOverviewPage } from './lazy-pages';
import { BOARD_PATH, OVERVIEW_PATH } from './routes.constants';

const AppRoutes = () => (
  <Suspense fallback={<PageFrame><LoadingFallback /></PageFrame>}>
    <Routes>
      <Route path={OVERVIEW_PATH} element={<LazyOverviewPage />} />
      <Route path={BOARD_PATH} element={<LazyBoardRoute />} />
      <Route path="*" element={<NotFoundPage />} />
    </Routes>
  </Suspense>
);

export { AppRoutes };
