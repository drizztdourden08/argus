/* @layer renderer-app @kind logic */
import { itemTypeOf } from './item-type-of';

const requiresParent = (type: string): boolean => itemTypeOf(type)?.parentRequired ?? false;

export { requiresParent };
