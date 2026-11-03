/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { nonEmpty } from '../../../shared';

const assigneeOf = (item: ItemRef): string => nonEmpty(item.assignee, item.thread);

export { assigneeOf };
