/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import type { TabField } from '../../../BoardTabsRow';
import type { Option } from '../../../shared';
import { KIND_OPTIONS } from '../worktrees.constants';
import { STATUS_LABEL } from './model.constants';
import type { WorktreeFilters, WorktreeRow } from './model.type';
import { threadOf } from './thread-of';

const distinct = (values: readonly string[]): string[] => [...new Set(values)].filter((value) => value !== '').sort();

const rowFields = (rows: readonly WorktreeRow[], meta: WorktreesMeta | null): TabField<WorktreeFilters>[] => {
  const threads: Option[] = distinct(rows.map((row) => threadOf(row, meta))).map((name) => ({ value: name, label: name }));
  const statuses: Option[] = distinct(rows.map((row) => row.status)).map((status) => ({ value: status, label: STATUS_LABEL[status] ?? status }));
  return [
    { kind: 'multi', path: 'threads', label: 'Thread', options: threads, read: (filters) => filters.threads, write: (values) => ({ threads: values }) },
    { kind: 'multi', path: 'statuses', label: 'Status', options: statuses, read: (filters) => filters.statuses, write: (values) => ({ statuses: values }) },
    { kind: 'multi', path: 'kinds', label: 'Kind', options: KIND_OPTIONS, read: (filters) => filters.kinds, write: (values) => ({ kinds: values }) },
  ];
};

export { rowFields };
