/* @layer renderer-app @kind logic */
import { itemTypeOf } from './item-type-of';
import type { ItemStage } from './registry.type';

const firstStageIn = (typeId: string, lane: string): ItemStage | undefined => itemTypeOf(typeId)?.stages.find((stage) => stage.lane === lane);

export { firstStageIn };
