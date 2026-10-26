/* @layer renderer-app @kind logic */
import type { StoredFilters } from '../shared.type';
import { jsonRecord } from './json-record';

const formatFilters = (existing: string | null, filters: StoredFilters, order: string): string => {
  const { query, conflict, ...rest } = filters;
  return JSON.stringify({ ...jsonRecord(existing), ...rest, q: query, conf: conflict, sort: order });
};

export { formatFilters };
