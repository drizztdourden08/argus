/* @layer renderer-app @kind logic */
import { ENDED_STAGES } from '../run.constants';
import type { PlanRun, RunResult } from '../run.type';
import { isRunStatus } from './is-run-status';
import type { Raw, RunSource } from './runs-of.type';

const record = (value: unknown): Raw | null => (typeof value === 'object' && value !== null && !Array.isArray(value) ? (value as Raw) : null);

const textOf = (value: unknown): string => (typeof value === 'string' ? value : '');

const resultOf = (value: unknown): RunResult | null => {
  const raw = record(value);
  const status = raw?.status;
  if (!raw || typeof raw.test !== 'string' || !isRunStatus(status)) return null;
  return { test: raw.test, status, note: textOf(raw.note), at: textOf(raw.at) };
};

const numberOf = (id: string): number => Number(id.replace(/^\D+/, ''));

const endedAt = (item: RunSource): string | null => {
  if (typeof item.endedAt === 'string') return item.endedAt;
  return ENDED_STAGES.includes(item.stage ?? '') ? item.updatedAt ?? '' : null;
};

const runOf = (item: RunSource): PlanRun => {
  const results = item.fields?.results;
  return {
    id: item.id,
    n: numberOf(item.id),
    build: textOf(item.fields?.build) || null,
    by: item.createdBy ?? '',
    startedAt: item.startedAt ?? item.createdAt ?? '',
    endedAt: endedAt(item),
    results: Array.isArray(results) ? results.map(resultOf).filter((result): result is RunResult => result !== null) : [],
    summary: textOf(item.fields?.summary),
  };
};

const isRunOf = (item: RunSource, planId: string): boolean =>
  item.type === 'test-run' && item.fields?.plan === planId && !item.deletedAt && item.stage !== 'abandoned';

const runsOf = (planId: string, items: readonly RunSource[]): PlanRun[] => items.filter((item) => isRunOf(item, planId)).map(runOf);

export { runsOf };
