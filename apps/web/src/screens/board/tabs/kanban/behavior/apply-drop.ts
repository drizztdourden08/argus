/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import type { ItemRef } from '../../../../../data/kanban';
import type { DropPlan } from '../model';
import { ORDER_HINT } from '../kanban.constants';
import type { ItemOps } from '../kanban.type';

const applyDrop = (plan: DropPlan, item: ItemRef, ops: ItemOps): void => {
  const { id } = item;
  switch (plan.kind) {
    case 'none':
      return;
    case 'order-off':
      toast({ message: ORDER_HINT, variant: 'info' });
      return;
    case 'reorder':
      void ops.move(item, plan.place, `Reordered ${id}.`);
      return;
    case 'status':
      void ops.move(item, { ...plan.place, status: plan.to }, `Moved ${id} to ${plan.to}.`);
      return;
    case 'thread':
      void ops.move(item, { ...plan.place, thread: plan.to }, `Reassigned ${id} to ${plan.to}.`);
      return;
    case 'subject':
      void ops.setField(item, 'subject', plan.to, `Subject of ${id} is now ${plan.to}.`);
  }
};

export { applyDrop };
