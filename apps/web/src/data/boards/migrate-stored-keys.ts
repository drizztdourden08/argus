/* @layer renderer-app @kind logic */
import type { BoardInfo } from './boards.type';
import { findBoard } from './find-board';

const idOfAlias = (boards: readonly BoardInfo[], key: string): string | null => {
  const board = findBoard(boards, key);
  return board && board.id !== key ? board.id : null;
};

const moveKey = (storage: Storage, from: string, to: string): void => {
  const value = storage.getItem(from);
  if (value === null) return;
  if (storage.getItem(to) === null) storage.setItem(to, value);
  storage.removeItem(from);
};

const migrateValue = (storage: Storage, boards: readonly BoardInfo[], key: string): void => {
  const id = idOfAlias(boards, storage.getItem(key) ?? '');
  if (id) storage.setItem(key, id);
};

const migrateScoped = (storage: Storage, boards: readonly BoardInfo[], prefix: string): void => {
  for (const board of boards) {
    for (const alias of board.aliases) moveKey(storage, `${prefix}.${alias}`, `${prefix}.${board.id}`);
  }
};

const migrateStoredKeys = (storage: Storage, boards: readonly BoardInfo[], valueKeys: readonly string[], scopedPrefixes: readonly string[]): boolean => {
  try {
    valueKeys.forEach((key) => migrateValue(storage, boards, key));
    scopedPrefixes.forEach((prefix) => migrateScoped(storage, boards, prefix));
    return true;
  } catch {
    return false;
  }
};

export { migrateStoredKeys };
