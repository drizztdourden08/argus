/* @layer renderer-app @kind logic */
import { jsonRecord } from '../../kanban/model/json-record';
import { DEFAULT_FILTERS, DEFAULT_PREFS, GROUP_OPTIONS } from './model.constants';
import type { RawStored, StoredView, TestFilters, TestGroup, TestOrder } from './model.type';

type Stored = Record<string, unknown>;

const textAt = (stored: Stored, key: string): string => {
  const value = stored[key];
  return typeof value === 'string' ? value : '';
};

const flagAt = (stored: Stored, key: string): boolean => stored[key] === true;

const listAt = (stored: Stored, key: string): string[] => {
  const value = stored[key];
  return Array.isArray(value) ? value.filter((entry): entry is string => typeof entry === 'string') : [];
};

const filtersOf = (stored: Stored): TestFilters => ({
  ...DEFAULT_FILTERS,
  query: textAt(stored, 'q'),
  threads: listAt(stored, 'threads'),
  builds: listAt(stored, 'builds'),
  nobuild: flagAt(stored, 'nobuild'),
  status: textAt(stored, 'status'),
  subject: textAt(stored, 'subject'),
  needs: flagAt(stored, 'needs'),
  conflict: flagAt(stored, 'conf'),
  archived: flagAt(stored, 'archived'),
});

const groupOf = (raw: string | null): TestGroup => GROUP_OPTIONS.find((option) => option.value === raw)?.value ?? DEFAULT_PREFS.group;

const orderOf = (stored: Stored): TestOrder => (textAt(stored, 'sort') === 'updated' ? 'updated' : DEFAULT_PREFS.order);

const parseStored = (raw: RawStored): StoredView => {
  const stored = jsonRecord(raw.filters);
  return {
    filters: filtersOf(stored),
    prefs: { group: groupOf(raw.group), order: orderOf(stored), showPassed: raw.passed !== '0', showClosed: raw.closed === '1' },
  };
};

export { parseStored };
