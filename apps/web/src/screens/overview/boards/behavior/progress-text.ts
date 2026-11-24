/* @layer renderer-app @kind logic */
import type { BoardProgress } from '../model';
import { WORDS } from '../boards-words.constants';

const progressText = (progress: BoardProgress): string => {
  const parts = [WORDS.progressDone(progress.done, progress.total)];
  if (progress.review > 0) parts.push(WORDS.progressReview(progress.review));
  if (progress.doing > 0) parts.push(WORDS.progressDoing(progress.doing));
  return parts.join(', ');
};

export { progressText };
