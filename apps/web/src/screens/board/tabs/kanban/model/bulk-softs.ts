/* @layer renderer-app @kind logic */
import { SOFT_INVERSE } from '../../../../../data/kanban';
import type { ItemRef, SoftKind } from '../../../../../data/kanban';
import type { BulkSofts } from './model.type';

const bulkSofts = (kind: SoftKind, items: readonly ItemRef[]): BulkSofts => ({
  ops: items.map((item) => ({ kind, target: { c: 'items', id: item.id } })),
  inverse: items.map((item) => ({ kind: SOFT_INVERSE[kind], target: { c: 'items', id: item.id } })),
});

export { bulkSofts };
