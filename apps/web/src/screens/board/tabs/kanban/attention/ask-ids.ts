/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { AskKind, AskLink } from '../model/model.type';
import { blockedOf } from './blocked-of';

const askIds = (item: ItemRef, asks: readonly AskLink[], kind: AskKind): string[] => {
  const linked = asks.filter((link) => link.kind === kind).map((link) => link.id);
  const blocking = blockedOf(item).filter((cause) => cause.kind === (kind === 'question' ? 'ask' : 'decision')).map((cause) => cause.id);
  return [...new Set([...linked, ...blocking])];
};

export { askIds };
