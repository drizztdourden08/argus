/* @layer renderer-app @kind test */
import { beforeAll } from 'vitest';
import { PAGE_LOADERS } from '../../src/routing/lazy-pages/loaders.constants';
import { BOARD_TAB_LOADERS } from '../../src/screens/board/BoardTabBody/behavior/loaders.constants';
import { OVERVIEW_TAB_LOADERS } from '../../src/screens/overview/lazy-tabs/loaders.constants';

const PRELOAD_TIMEOUT_MS = 300000;

const loaders = [...Object.values(PAGE_LOADERS), ...Object.values(BOARD_TAB_LOADERS), ...Object.values(OVERVIEW_TAB_LOADERS)];

const preloadRoutes = (): void => {
  beforeAll(async () => {
    await Promise.all(loaders.map((load) => load()));
  }, PRELOAD_TIMEOUT_MS);
};

export { preloadRoutes };
