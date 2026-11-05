/* @layer renderer-app @kind logic */
import { POINTER } from './backdrop.constants';

const easeStep = (
  current: number,
  target: number,
  seconds: number,
  settleSeconds: number = POINTER.settleSeconds,
): number => {
  if (seconds <= 0) return current;
  const follow = 1 - Math.exp((-3 * seconds) / settleSeconds);
  return current + (target - current) * follow;
};

export { easeStep };
