/* @layer renderer-app @kind test */
import { beforeAll } from 'vitest';
import { loadBoardRoute, loadOverviewPage } from '../../src/routing/lazy-pages';
import {
  loadActivityTab as loadBoardActivityTab,
  loadAgentsTab,
  loadBuildsTab,
  loadChangesTab,
  loadKanbanTab,
  loadReleasesTab as loadBoardReleasesTab,
  loadTestsTab,
  loadWorktreesTab,
} from '../../src/screens/board/BoardTabBody/lazy-tabs';
import {
  loadActivityTab as loadOverviewActivityTab,
  loadCacheTab,
  loadDashboardsTab,
  loadDependenciesTab,
  loadNotificationsTab,
  loadProcessesTab,
  loadRegistryTab,
  loadReleasesTab as loadOverviewReleasesTab,
} from '../../src/screens/overview/lazy-tabs';

// The first import of a route pulls the Tessera source barrel (about 5000 modules). Under the
// heavy-check lock that took 45 to 90 s, longer than any wait in a test. The routes and tabs are
// loaded here once per file, before the first test, so React.lazy finds them in the module cache.
// The hook gets its own timeout: the standards hookTimeout is 30 s, too short for that import.
const PRELOAD_TIMEOUT_MS = 300000;

const loaders = [
  loadBoardRoute,
  loadOverviewPage,
  loadBoardActivityTab,
  loadAgentsTab,
  loadBuildsTab,
  loadChangesTab,
  loadKanbanTab,
  loadBoardReleasesTab,
  loadTestsTab,
  loadWorktreesTab,
  loadOverviewActivityTab,
  loadCacheTab,
  loadDashboardsTab,
  loadDependenciesTab,
  loadNotificationsTab,
  loadProcessesTab,
  loadRegistryTab,
  loadOverviewReleasesTab,
];

const preloadRoutes = (): void => {
  beforeAll(async () => {
    await Promise.all(loaders.map((load) => load()));
  }, PRELOAD_TIMEOUT_MS);
};

export { preloadRoutes };
