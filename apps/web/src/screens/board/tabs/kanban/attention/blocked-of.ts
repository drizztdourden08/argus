/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { BlockedCause } from '../../../../../data/types';
import { NONE } from './blocked-of.constants';

const blockedOf = (item: ItemRef): readonly BlockedCause[] => item.blocked ?? NONE;

export { blockedOf };
