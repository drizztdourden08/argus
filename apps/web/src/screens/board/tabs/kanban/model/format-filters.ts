/* @layer renderer-app @kind logic */
import { jsonRecord } from './json-record';
import type { ItemOrder, KanbanFilters } from './model.type';

const formatFilters = (existing: string | null, filters: KanbanFilters, order: ItemOrder): string => {
  const { query, conflict, ...rest } = filters;
  return JSON.stringify({ ...jsonRecord(existing), ...rest, q: query, conf: conflict, sort: order });
};

export { formatFilters };
