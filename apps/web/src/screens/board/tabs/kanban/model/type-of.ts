/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { DEFAULT_TYPE } from './model.constants';

const typeOf = (item: ItemRef): string => item.type || DEFAULT_TYPE;

export { typeOf };
