/* @layer renderer-app @kind constants */
const OVERVIEW_TAB_LOADERS = {
  accounts: () => import('../accounts/AccountsTab').then((loaded) => ({ default: loaded.AccountsTab })),
  activity: () => import('../activity/ActivityTab').then((loaded) => ({ default: loaded.ActivityTab })),
  cache: () => import('../cache/CacheTab').then((loaded) => ({ default: loaded.CacheTab })),
  dependencies: () => import('../deps/DepsTab').then((loaded) => ({ default: loaded.DepsTab })),
  notifications: () => import('../notifications/NotificationsTab').then((loaded) => ({ default: loaded.NotificationsTab })),
  overlord: () => import('../overlord/OverlordTab').then((loaded) => ({ default: loaded.OverlordTab })),
  processes: () => import('../processes/ProcessesTab').then((loaded) => ({ default: loaded.ProcessesTab })),
  registry: () => import('../registry/RegistryTab').then((loaded) => ({ default: loaded.RegistryTab })),
  releases: () => import('../releases/ReleasesTab').then((loaded) => ({ default: loaded.ReleasesTab })),
};

export { OVERVIEW_TAB_LOADERS };
