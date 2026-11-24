/* @layer renderer-app @kind constants */
const OVERVIEW_TAB_LOADERS = {
  accounts: () => import('../accounts/AccountsTab').then((loaded) => ({ default: loaded.AccountsTab })),
  activity: () => import('../activity/ActivityTab').then((loaded) => ({ default: loaded.ActivityTab })),
  dependencies: () => import('../deps/DepsTab').then((loaded) => ({ default: loaded.DepsTab })),
  notifications: () => import('../notifications/NotificationsTab').then((loaded) => ({ default: loaded.NotificationsTab })),
  processes: () => import('../processes/ProcessesTab').then((loaded) => ({ default: loaded.ProcessesTab })),
  packages: () => import('../packages/PackagesTab').then((loaded) => ({ default: loaded.PackagesTab })),
  releases: () => import('../releases/ReleasesTab').then((loaded) => ({ default: loaded.ReleasesTab })),
};

export { OVERVIEW_TAB_LOADERS };
