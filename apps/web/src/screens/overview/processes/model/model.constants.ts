/* @layer renderer-app @kind constants */
import type { FieldDescriptor, TableColumn, TagUrgencyColor } from '@drizztdourden08/tessera';

const STATUS_TIP: Readonly<Record<string, string>> = {
  ok: 'Within its rule.',
  hog: 'Uses more memory than its rule allows.',
  stale: 'Idle for longer than its rule allows.',
  orphan: 'Its parent process is gone.',
  hung: 'Not responding.',
  old: 'Running for longer than its rule allows.',
};

const STATUS_COLOR: Readonly<Record<string, TagUrgencyColor>> = {
  ok: 'success',
  hog: 'warning',
  stale: 'warning',
  orphan: 'warning',
  hung: 'danger',
  old: 'info',
};

const STATUS_OPTIONS = ['ok', 'hog', 'stale', 'orphan', 'hung', 'old'] as const;

const PROC_SCHEMA: readonly FieldDescriptor[] = [
  { path: 'rule', label: 'Rule', kind: 'string', optional: false },
  { path: 'name', label: 'Process', kind: 'string', optional: false },
  { path: 'pid', label: 'PID', kind: 'number', optional: true },
  { path: 'project', label: 'Project', kind: 'string', optional: true },
  { path: 'memMB', label: 'Memory (MB)', kind: 'number', optional: true },
  { path: 'cpuPct', label: 'CPU (%)', kind: 'number', optional: true },
  { path: 'ageMin', label: 'Age (min)', kind: 'number', optional: true },
  { path: 'idleMin', label: 'Idle (min)', kind: 'number', optional: true },
  { path: 'status', label: 'Status', kind: 'enum', optional: false, closed: true, options: STATUS_OPTIONS },
  { path: 'cmd', label: 'Command', kind: 'string', optional: true },
  { path: 'kill', label: 'Kill', kind: 'string', optional: true },
];

const CUSTOM_PATHS: readonly string[] = ['project', 'status', 'cmd', 'kill'];

const PROC_COLUMNS: readonly TableColumn[] = PROC_SCHEMA.map((field) => ({ path: field.path }));

export { CUSTOM_PATHS, PROC_COLUMNS, PROC_SCHEMA, STATUS_COLOR, STATUS_TIP };
