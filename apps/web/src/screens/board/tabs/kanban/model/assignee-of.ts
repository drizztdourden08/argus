/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const assigneeOf = (item: ItemRef): string => item.assignee || item.thread;

export { assigneeOf };
