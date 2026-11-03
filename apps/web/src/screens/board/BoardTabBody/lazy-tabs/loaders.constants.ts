/* @layer renderer-app @kind constants */
const BOARD_TAB_LOADERS = {
  activity: () => import('../../tabs/activity').then((loaded) => ({ default: loaded.ActivityTab })),
  agents: () => import('../../tabs/agents').then((loaded) => ({ default: loaded.AgentsTab })),
  builds: () => import('../../tabs/builds').then((loaded) => ({ default: loaded.BuildsTab })),
  changes: () => import('../../tabs/changes').then((loaded) => ({ default: loaded.ChangesTab })),
  kanban: () => import('../../tabs/kanban').then((loaded) => ({ default: loaded.KanbanTab })),
  media: () => import('../../tabs/media').then((loaded) => ({ default: loaded.MediaTab })),
  releases: () => import('../../tabs/releases').then((loaded) => ({ default: loaded.ReleasesTab })),
  tests: () => import('../../tabs/tests').then((loaded) => ({ default: loaded.TestsTab })),
  worktrees: () => import('../../tabs/worktrees').then((loaded) => ({ default: loaded.WorktreesTab })),
};

export { BOARD_TAB_LOADERS };
