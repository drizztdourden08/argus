/* @layer renderer-app @kind logic */
import { REORDER_ITEM_SELECTOR, reorderStep, useMoveFocus, useReorderDrag } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import type { KeyboardEvent } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import { criterionKeys } from '../model';
import type { CriteriaReorder, ItemOps } from '../kanban.type';

const useCriteriaReorder = (item: ItemRef, ops: ItemOps): CriteriaReorder => {
  const listRef = useRef<HTMLElement | null>(null);
  const focus = useMoveFocus(listRef);
  const criteria = item.ac ?? [];
  const keys = criterionKeys(criteria.map((criterion) => criterion.text));
  const move = (from: number, to: number) => {
    focus.announce(`Criterion ${from + 1} moved to ${to + 1} of ${criteria.length}`);
    void ops.criterion(item, { do: 'move', n: from + 1, to: to + 1 }, `Reordered the criteria of ${item.id}.`);
  };
  const drag = useReorderDrag({ listRef, selector: REORDER_ITEM_SELECTOR, axis: 'vertical', onDrop: move });
  const step = (event: KeyboardEvent<HTMLElement>, index: number) => {
    if (event.key === 'Escape') drag.cancel();
    const to = reorderStep(event, { axis: 'vertical', chord: 'plain', index, total: criteria.length });
    if (to === undefined) return;
    event.preventDefault();
    focus.expect(keys[index] ?? '');
    move(index, to);
  };
  return { listRef, drag, keys, message: focus.message, step };
};

export { useCriteriaReorder };
