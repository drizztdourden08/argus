/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import type { Decisions, TotalLine } from '../review.type';
import { choiceOf } from './choice-of';

const choiceTotals = (rows: readonly ReviewRow[], decisions: Decisions): TotalLine[] => {
  const counts = new Map<string, number>();
  for (const row of rows) {
    if (!row.diff) continue;
    const id = choiceOf(row, decisions);
    const label = row.diff.choices.find((choice) => choice.id === id)?.label ?? id;
    counts.set(label, (counts.get(label) ?? 0) + 1);
  }
  return [...counts].map(([label, count]) => ({ label, count }));
};

export { choiceTotals };
