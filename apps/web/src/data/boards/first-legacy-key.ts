/* @layer renderer-app @kind logic */
import { boardsStore } from './boards-store';
import { legacyKeys } from './legacy-keys';
import type { BoardInfo } from './boards.type';

const firstLegacyKey = (key: string, boards: readonly BoardInfo[] = boardsStore.get()): string => legacyKeys(key, boards)[1] ?? key;

export { firstLegacyKey };
