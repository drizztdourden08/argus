/* @layer renderer-app @kind constants */
const PAGE_LOADERS = {
  boardMode: () => import('../../screens/board/BoardMode').then((loaded) => ({ default: loaded.BoardMode })),
  overviewMode: () => import('../../screens/overview/OverviewMode').then((loaded) => ({ default: loaded.OverviewMode })),
};

export { PAGE_LOADERS };
