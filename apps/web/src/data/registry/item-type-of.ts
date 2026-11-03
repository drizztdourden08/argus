/* @layer renderer-app @kind logic */
import { ITEM_REGISTRY } from './registry.constants';
import type { ItemType } from './registry.type';

const itemTypeOf = (id: string): ItemType | undefined => ITEM_REGISTRY.types.find((type) => type.id === id);

export { itemTypeOf };
