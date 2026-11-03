/* @layer renderer-app @kind logic */
import { ITEM_HASH } from './model.constants';

const hashTab = (hash: string): string | null => ITEM_HASH.exec(hash.replace(/^#/, ''))?.[2] ?? null;

export { hashTab };
