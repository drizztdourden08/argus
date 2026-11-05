/* @layer renderer-app @kind logic */
import type { BoardProgress, BoardRecord } from './board.type';

const boardProgress = (record: BoardRecord): BoardProgress => {
  const { done, review, doing, itemsOpen } = record.counts;
  const todo = Math.max(0, itemsOpen - review - doing);
  const total = done + itemsOpen;
  return { done, review, doing, todo, total, ratio: total === 0 ? 0 : done / total };
};

export { boardProgress };
