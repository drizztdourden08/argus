/* @layer renderer-app @kind logic */
import type { Counts } from './model.type';

const SETTLED = ['pass', 'fail', 'partial', 'blocked'] as const;

const lastStatuses = (results: unknown): Map<string, string> => {
  const last = new Map<string, string>();
  if (!Array.isArray(results)) return last;
  for (const entry of results) {
    const { test, status } = (entry ?? {}) as Record<string, unknown>;
    if (typeof test === 'string' && typeof status === 'string') last.set(test, status);
  }
  return last;
};

const runCounts = (results: unknown, cases: readonly string[]): Counts => {
  const last = lastStatuses(results);
  const counts: Counts = { run: 0, pass: 0, fail: 0, partial: 0, blocked: 0 };
  for (const id of cases) {
    const status = SETTLED.find((candidate) => candidate === last.get(id));
    if (status) counts[status] += 1;
    else counts.run += 1;
  }
  return counts;
};

export { runCounts };
