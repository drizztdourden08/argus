/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const blockedSignature = (items: readonly ItemRef[]): string =>
  items
    .filter((item) => (item.blocked?.length ?? 0) > 0 && !item.deletedAt)
    .map((item) => `${item.id}:${(item.blocked ?? []).map((cause) => cause.id).join(',')}`)
    .join('|');

export { blockedSignature };
