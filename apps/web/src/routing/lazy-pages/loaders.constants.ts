/* @layer renderer-app @kind constants */
const PAGE_LOADERS = {
  boardRoute: () => import('../../screens/board/BoardRoute').then((loaded) => ({ default: loaded.BoardRoute })),
  overviewPage: () => import('../../screens/overview/OverviewPage').then((loaded) => ({ default: loaded.OverviewPage })),
};

export { PAGE_LOADERS };
