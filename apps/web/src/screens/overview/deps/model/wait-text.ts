/* @layer renderer-app @kind logic */
import type { DepDoc } from '../../../../data/deps';
import { formatAgo } from '../../../../format';
import { startedAt } from './started-at';
import { waitMs } from './wait-ms';

const waitText = (doc: DepDoc, now: number): string => {
  const start = startedAt(doc);
  return start === null ? 'never' : formatAgo(start, Date.parse(start) + waitMs(doc, now));
};

export { waitText };
