/* @layer renderer-app @kind logic */
import { ownEntry } from './own-entry';
import { ITEM_REGISTRY } from './registry.constants';
import type { ItemField } from './registry.type';

const fieldOf = (id: string): ItemField | undefined => ownEntry(ITEM_REGISTRY.fields, id);

export { fieldOf };
