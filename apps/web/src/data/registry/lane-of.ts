/* @layer renderer-app @kind logic */
import { itemTypeOf } from './item-type-of';

const laneOf = (typeId: string, stageId: string): string | undefined => itemTypeOf(typeId)?.stages.find((stage) => stage.id === stageId)?.lane;

export { laneOf };
