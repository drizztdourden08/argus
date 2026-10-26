/* @layer renderer-app @kind logic */
import { isLeftover } from './is-leftover';
import { isNeeding } from './is-needing';
import type { Counts, WorktreeRow } from './model.type';

const countRows = (rows: readonly WorktreeRow[]): Counts => {
  const left = rows.filter(isLeftover).length;
  return { all: rows.length, need: rows.filter(isNeeding).length, left, git: rows.length - left };
};

export { countRows };
