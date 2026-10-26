/* @layer renderer-app @kind component */
import { Route, Routes } from 'react-router';
import { BoardRoute } from '../screens/board/BoardRoute';
import { NotFoundPage } from '../screens/not-found/NotFoundPage';
import { OverviewPage } from '../screens/overview/OverviewPage';
import { BOARD_PATH, OVERVIEW_PATH } from './routes.constants';

const AppRoutes = () => (
  <Routes>
    <Route path={OVERVIEW_PATH} element={<OverviewPage />} />
    <Route path={BOARD_PATH} element={<BoardRoute />} />
    <Route path="*" element={<NotFoundPage />} />
  </Routes>
);

export { AppRoutes };
