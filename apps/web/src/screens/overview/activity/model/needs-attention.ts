/* @layer renderer-app @kind logic */
import { levelOf } from './level-of';
import type { ActivityItem } from './model.type';

const needsAttention = (item: ActivityItem): boolean => levelOf(item) !== 'info';

export { needsAttention };
