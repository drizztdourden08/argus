/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { RUN_WORDS } from './model.constants';
import type { CardProgress } from './model.type';
import type { Row } from './row.type';

const rowsOf = (value: unknown): Row[] => (Array.isArray(value) ? value.filter((row): row is Row => typeof row === 'object' && row !== null) : []);

const numberOf = (row: Row): number => (typeof row.n === 'number' ? row.n : 0);

const lastRun = (runs: readonly Row[]): Row | undefined => runs.reduce<Row | undefined>((last, run) => (last && numberOf(last) > numberOf(run) ? last : run), undefined);

const wordsOf = (results: readonly Row[], left: number): string[] => [
  ...RUN_WORDS.map((word) => ({ word, count: results.filter((result) => result.status === word).length })).filter(({ count }) => count > 0).map(({ word, count }) => `${count} ${word}`),
  ...(left > 0 ? [`${left} left`] : []),
];

const runProgress = (item: ItemRef): CardProgress | null => {
  const run = lastRun(rowsOf(item.fields?.runs));
  if (!run) return null;
  const results = rowsOf(run.results);
  const cases = Array.isArray(item.fields?.cases) ? item.fields.cases.length : 0;
  const total = Math.max(cases, results.length);
  const text = [`run ${numberOf(run)}:`, ...wordsOf(results, total - results.length)].join(' ');
  return { met: results.length, total, text, title: `${results.length} of ${total} cases have a result in run ${numberOf(run)}` };
};

export { runProgress };
