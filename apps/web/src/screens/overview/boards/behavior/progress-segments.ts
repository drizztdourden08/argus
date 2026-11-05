/* @layer renderer-app @kind logic */
import type { CardProgress } from '../../../board/tabs/kanban/model';
import type { BoardProgress } from '../model';

const progressSegments = (progress: BoardProgress): CardProgress[] => {
  if (progress.total === 0) return [];
  return [
    { met: progress.done, total: progress.total, text: `${progress.done}/${progress.total}`, title: 'Items done' },
    { met: 0, total: 0, text: `${progress.review} review`, title: 'Items in review' },
    { met: 0, total: 0, text: `${progress.doing} doing`, title: 'Items in progress' },
  ];
};

export { progressSegments };
