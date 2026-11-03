/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { nonEmpty } from '../../../shared';
import { DEFAULT_TYPE } from './model.constants';

const typeOf = (item: ItemRef): string => nonEmpty(item.type, DEFAULT_TYPE);

export { typeOf };
