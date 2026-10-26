/* @layer renderer-app @kind logic */
import { toMillis } from '../format';

const isNewer = (at: unknown, since: number): boolean => {
  const millis = toMillis(at);
  return Number.isFinite(millis) && (!Number.isFinite(since) || millis > since);
};

export { isNewer };
