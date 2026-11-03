/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { BlockedCause } from '../../../../../data/types';

const NONE: readonly BlockedCause[] = [];

const blockedOf = (item: ItemRef): readonly BlockedCause[] => item.blocked ?? NONE;

export { blockedOf };
