/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { itemTypeOf } from '../../../../../data/registry';
import { typeOf } from '../../kanban/model';

const stageText = (item: ItemRef): string => {
  const stages = itemTypeOf(typeOf(item))?.stages ?? [];
  return stages.find((stage) => stage.id === item.stage)?.label ?? item.stage ?? item.status;
};

export { stageText };
