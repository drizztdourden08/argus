/* @layer renderer-app @kind logic */
import { resetClock } from './reset-clock';

const resetText = (resetsAt: string | null | undefined, now: number): string => {
  const clock = resetClock(resetsAt, now);
  return clock ? `resets ${clock}` : '';
};

export { resetText };
