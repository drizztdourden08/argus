/* @layer renderer-app @kind logic */
import { ITEM_HASH } from './model.constants';

const itemHash = (hash: string): string | null => ITEM_HASH.exec(hash.replace(/^#/, ''))?.[1] ?? null;

export { itemHash };
