/* @layer renderer-app @kind logic */
import { isLeftover } from './is-leftover';
import { isSafe } from './is-safe';
import { REMOVE_LOOKS } from './model.constants';
import type { RemoveLook, WorktreeRow } from './model.type';

const removeLook = (row: WorktreeRow): RemoveLook => {
  if (isLeftover(row)) return REMOVE_LOOKS.leftover;
  return isSafe(row) ? REMOVE_LOOKS.safe : REMOVE_LOOKS.discard;
};

export { removeLook };
