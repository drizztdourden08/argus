/* @layer renderer-app @kind logic */
import { toMillis } from '../../../format';
import type { Timed } from './seen.type';

const newestAt = (items: readonly Timed[]): number =>
  items.reduce((newest, item) => {
    const at = toMillis(item.at);
    return Number.isFinite(at) ? Math.max(newest, at) : newest;
  }, 0);

export { newestAt };
