/* @layer renderer-app @kind logic */
import type { DepHistoryRow } from '../../../../../data/deps';
import { agoText } from '../../../../../format';

const historyText = (row: DepHistoryRow, now: number): string => {
  const move = `${row.by} moved it from ${row.from ?? 'nothing'} to ${row.to}`;
  return [agoText(row.at, now), row.reason ? `${move}, ${row.reason}` : move].filter(Boolean).join(': ');
};

export { historyText };
