/* @layer renderer-app @kind logic */
import { CLOCK_SECONDS, RESET_TIME } from '../accounts.constants';

const clockText = (at: string | number | null | undefined, seconds = false): string => {
  if (at === null || at === undefined || at === '') return '';
  const date = new Date(at);
  if (Number.isNaN(date.getTime())) return '';
  return date.toLocaleTimeString('en-CA', seconds ? CLOCK_SECONDS : RESET_TIME);
};

export { clockText };
