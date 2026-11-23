/* @layer renderer-app @kind logic */
import { DAY_MS, RESET_DAY, RESET_TIME } from '../accounts.constants';

const resetClock = (resetsAt: string | null | undefined, now: number): string => {
  const at = resetsAt ? new Date(resetsAt) : null;
  if (!at || Number.isNaN(at.getTime())) return '';
  return at.toLocaleString('en-CA', at.getTime() - now < DAY_MS ? RESET_TIME : RESET_DAY);
};

export { resetClock };
