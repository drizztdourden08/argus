/* @layer renderer-app @kind logic */
import type { KanbanColumnConfig, KanbanConfig } from '../../../../../data/types';
import { DEFAULT_QUIET_MS } from './quiet-limits.constants';

const dwellOf = (config: KanbanConfig, column: KanbanColumnConfig): number | null => {
  const rule = (config.rules ?? []).find((candidate) => candidate.kind === 'dwell' && (column.rules ?? []).includes(candidate.id));
  const ms = rule?.params?.ms;
  return typeof ms === 'number' && ms > 0 ? ms : null;
};

const quietLimits = (config: KanbanConfig | null | undefined, columns: readonly KanbanColumnConfig[] | null): Readonly<Record<string, number>> => {
  if (!config || !columns) return DEFAULT_QUIET_MS;
  const entries = columns.flatMap((column) => {
    const ms = dwellOf(config, column);
    return ms === null ? [] : column.match.lanes.map((lane) => [lane, ms] as const);
  });
  return Object.fromEntries([...entries].reverse());
};

export { quietLimits };
