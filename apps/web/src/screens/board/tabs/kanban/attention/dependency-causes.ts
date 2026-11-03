/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { BlockedCause } from '../../../../../data/types';
import { DEPENDENCY_KINDS } from './attention.constants';
import { blockedOf } from './blocked-of';

const dependencyCauses = (item: ItemRef): BlockedCause[] => blockedOf(item).filter((cause) => DEPENDENCY_KINDS.includes(cause.kind));

export { dependencyCauses };
