/* @layer renderer-app @kind logic */
import { ITEM_HASH } from './model.constants';

const itemHash = (hash: string): string | null => {
  const wanted = hash.replace(/^#/, '');
  return ITEM_HASH.test(wanted) ? wanted : null;
};

export { itemHash };
