/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../../data/kanban';
import { EMPTY_SUMMARY, SUMMARY_ROWS } from '../run.constants';
import type { PlanRun, RunResult, RunSummary } from '../run.type';
import { isRunStatus } from './is-run-status';
import type { Raw } from './runs-of.type';

const record = (value: unknown): Raw | null => (typeof value === 'object' && value !== null && !Array.isArray(value) ? (value as Raw) : null);

const textOf = (value: unknown): string => (typeof value === 'string' ? value : '');

const resultOf = (value: unknown): RunResult | null => {
  const raw = record(value);
  const status = raw?.status;
  if (!raw || typeof raw.test !== 'string' || !isRunStatus(status)) return null;
  return { test: raw.test, status, note: textOf(raw.note), at: textOf(raw.at) };
};

const countOf = (raw: Raw, key: string): number => {
  const value = raw[key];
  return typeof value === 'number' ? value : 0;
};

const summaryOf = (value: unknown): RunSummary | null => {
  const raw = record(value);
  if (!raw || typeof raw.total !== 'number') return null;
  const counts = { ...EMPTY_SUMMARY, total: raw.total };
  SUMMARY_ROWS.forEach(([key]) => {
    counts[key] = countOf(raw, key);
  });
  return counts;
};

const runOf = (value: unknown): PlanRun | null => {
  const raw = record(value);
  if (!raw || typeof raw.n !== 'number') return null;
  const results = Array.isArray(raw.results) ? raw.results.map(resultOf).filter((result): result is RunResult => result !== null) : [];
  return {
    n: raw.n,
    bundle: typeof raw.bundle === 'string' ? raw.bundle : null,
    by: textOf(raw.by),
    startedAt: textOf(raw.startedAt),
    endedAt: typeof raw.endedAt === 'string' ? raw.endedAt : null,
    results,
    summary: summaryOf(raw.summary),
  };
};

const runsOf = (item: ItemRef): PlanRun[] => {
  const raw = item.fields?.runs;
  return Array.isArray(raw) ? raw.map(runOf).filter((run): run is PlanRun => run !== null) : [];
};

export { runsOf };
