/* @layer renderer-app @kind logic */
import { jsonRecord } from './json-record';
import { DEFAULT_FILTERS, DEFAULT_PREFS, GROUP_OPTIONS } from './model.constants';
import type { ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs, RawStored, StoredView } from './model.type';

const text = (value: unknown): string => (typeof value === 'string' ? value : '');

const flag = (value: unknown): boolean => value === true;

const strings = (value: unknown): string[] => (Array.isArray(value) ? value.filter((entry): entry is string => typeof entry === 'string') : []);

const filtersOf = (stored: Record<string, unknown>): KanbanFilters => ({
  ...DEFAULT_FILTERS,
  query: text(stored.q),
  threads: strings(stored.threads),
  status: text(stored.status),
  subject: text(stored.subject),
  needs: flag(stored.needs),
  conflict: flag(stored.conf),
  archived: flag(stored.archived),
});

const groupOf = (raw: string | null): ItemGroup => GROUP_OPTIONS.find((option) => option.value === raw)?.value ?? DEFAULT_PREFS.group;

const orderOf = (stored: Record<string, unknown>): ItemOrder => (stored.sort === 'updated' ? 'updated' : DEFAULT_PREFS.order);

const parseStored = (raw: RawStored): StoredView => {
  const stored = jsonRecord(raw.filters);
  const prefs: KanbanPrefs = { group: groupOf(raw.group), order: orderOf(stored), showDone: raw.showDone === '1' };
  return { filters: filtersOf(stored), prefs };
};

export { parseStored };
