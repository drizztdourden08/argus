/* @layer renderer-app @kind logic */
import { itemTypeOf } from './item-type-of';

const allowsParent = (child: string, parent: string): boolean => itemTypeOf(child)?.parents.includes(parent) ?? false;

export { allowsParent };
