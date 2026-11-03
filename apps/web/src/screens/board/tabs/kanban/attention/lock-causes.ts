/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { BlockedCause } from '../../../../../data/types';
import type { AskLink } from '../model/model.type';
import { LOCK_KINDS } from './attention.constants';
import { blockedOf } from './blocked-of';

const namesAsk = (cause: BlockedCause, asks: readonly AskLink[]): boolean =>
  cause.kind === 'legacy' && asks.some((link) => link.via === 'blocker' && link.id === cause.label.trim());

const lockCauses = (item: ItemRef, asks: readonly AskLink[]): BlockedCause[] =>
  blockedOf(item).filter((cause) => LOCK_KINDS.includes(cause.kind) && !namesAsk(cause, asks));

export { lockCauses };
