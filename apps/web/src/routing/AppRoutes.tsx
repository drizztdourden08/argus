/* @layer renderer-app @kind component */
import { Route, Routes } from 'react-router';
import { NotFoundPage } from '../screens/not-found/NotFoundPage';
import { AppShell } from '../shell/AppShell';
import { LazyBoardMode, LazyOverviewMode } from './lazy-pages';
import { BOARD_PATH } from './routes.constants';

const AppRoutes = () => (
  <Routes>
    <Route element={<AppShell />}>
      <Route index element={<LazyOverviewMode />} />
      <Route path={BOARD_PATH} element={<LazyBoardMode />} />
    </Route>
    <Route path="*" element={<NotFoundPage />} />
  </Routes>
);

export { AppRoutes };
