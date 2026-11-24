/* @layer renderer-app @kind logic */
import type { Direction } from '../next.type';

const copyDirection = (direction: Direction, left: string, right: string): { from: string; to: string } | null => {
  if (direction === 'ltr') return { from: left, to: right };
  if (direction === 'rtl') return { from: right, to: left };
  return null;
};

export { copyDirection };
