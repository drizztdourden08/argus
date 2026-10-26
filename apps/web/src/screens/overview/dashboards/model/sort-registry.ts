/* @layer renderer-app @kind logic */
import { REGISTRY_RANK, REGISTRY_RANK_FALLBACK } from './model.constants';
import type { RegistryItem } from './model.type';

const rank = (row: RegistryItem): number => REGISTRY_RANK[row.state] ?? REGISTRY_RANK_FALLBACK;

const sortRegistry = (rows: readonly RegistryItem[], nameOf: (row: RegistryItem) => string): RegistryItem[] =>
  [...rows].sort((a, b) => rank(a) - rank(b) || nameOf(a).localeCompare(nameOf(b)));

export { sortRegistry };
