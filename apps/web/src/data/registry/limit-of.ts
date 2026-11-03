/* @layer renderer-app @kind logic */
import { ownEntry } from './own-entry';
import { ITEM_REGISTRY } from './registry.constants';

const limitOf = (kind: string): number | undefined => ownEntry(ITEM_REGISTRY.limits, kind);

export { limitOf };
