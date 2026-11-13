/* @layer renderer-app @kind logic */
import { boardsStore } from './boards-store';
import { findBoard } from './find-board';
import type { BoardInfo } from './boards.type';

const legacyKeys = (key: string, boards: readonly BoardInfo[] = boardsStore.get()): string[] => [
  key,
  ...(findBoard(boards, key)?.aliases ?? []).filter((alias) => alias !== key),
];

export { legacyKeys };
