/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { firstStageIn, itemTypeOf } from '../../../../../data/registry';
import type { ItemStage } from '../../../../../data/registry';
import { typeOf } from './type-of';
import type { ItemGroup } from './model.type';

const stageOf = (item: ItemRef): ItemStage | undefined => {
  const type = typeOf(item);
  return itemTypeOf(type)?.stages.find((stage) => stage.id === item.stage) ?? firstStageIn(type, item.status);
};

const stageLabel = (item: ItemRef, group: ItemGroup): string | null => {
  const stage = stageOf(item);
  if (group === 'status') return stage && stage.id !== stage.lane ? stage.label : null;
  return stage?.label ?? item.status;
};

export { stageLabel };
