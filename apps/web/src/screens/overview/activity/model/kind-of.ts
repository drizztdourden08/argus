/* @layer renderer-app @kind logic */
import type { ActivityItem } from './model.type';

const kindOf = (item: ActivityItem): string => (typeof item.kind === 'string' && item.kind ? item.kind : 'note');

export { kindOf };
