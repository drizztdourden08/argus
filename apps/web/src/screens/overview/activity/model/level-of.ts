/* @layer renderer-app @kind logic */
import type { ActivityLevel } from '../../../../data/types';
import { LEVELS } from './model.constants';
import type { ActivityItem } from './model.type';

const levelOf = (item: ActivityItem): ActivityLevel =>
  LEVELS.find((level) => level === item.level) ?? 'info';

export { levelOf };
