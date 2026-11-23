/* @layer renderer-app @kind logic */
import type { BoardProgress } from '../model';

const progressText = (progress: BoardProgress): string => {
  const parts = [`${progress.done} of ${progress.total} done`];
  if (progress.review > 0) parts.push(`${progress.review} to review`);
  if (progress.doing > 0) parts.push(`${progress.doing} in progress`);
  return parts.join(', ');
};

export { progressText };
