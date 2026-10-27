/* @layer renderer-app @kind constants */
import type { FieldDescriptor, StatusTone, TableColumn } from '@drizztdourden08/tessera';

const STATUS_TIP: Readonly<Record<string, string>> = {
  ok: 'Within its rule.',
  hog: 'Uses more memory than its rule allows.',
  stale: 'Idle for longer than its rule allows.',
  orphan: 'Its parent process is gone.',
  hung: 'Not responding.',
  old: 'Running for longer than its rule allows.',
};

const STATUS_TONE: Readonly<Record<string, StatusTone>> = {
  ok: 'success',
  hog: 'warning',
  stale: 'warning',
  old: 'warning',
  orphan: 'danger',
  hung: 'danger',
};

const STATUS_OPTIONS = ['ok', 'hog', 'stale', 'orphan', 'hung', 'old'] as const;

const PROC_SCHEMA: readonly FieldDescriptor[] = [
  { path: 'rule', label: 'Rule', kind: 'string', optional: false },
  { path: 'name', label: 'Process', kind: 'string', optional: false },
  { path: 'pid', label: 'PID', kind: 'number', optional: true },
  { path: 'project', label: 'Project', kind: 'string', optional: true },
  { path: 'memMB', label: 'Memory', kind: 'number', optional: true },
  { path: 'cpuPct', label: 'CPU', kind: 'number', optional: true, unit: '%' },
  { path: 'ageMin', label: 'Age', kind: 'number', optional: true },
  { path: 'idleMin', label: 'Idle', kind: 'number', optional: true },
  { path: 'status', label: 'Status', kind: 'enum', optional: false, closed: true, options: STATUS_OPTIONS },
  { path: 'cmd', label: 'Command', kind: 'string', optional: true },
  { path: 'kill', label: 'Kill', kind: 'string', optional: true },
];

const CUSTOM_PATHS: readonly string[] = ['name', 'project', 'memMB', 'cpuPct', 'ageMin', 'idleMin', 'status', 'cmd', 'kill'];

const COLUMN_SIZE: Readonly<Record<string, Omit<TableColumn, 'path'>>> = {
  rule: { width: 140 },
  name: { width: 190 },
  project: { width: 100 },
  memMB: { width: 84 },
  cpuPct: { width: 64 },
  ageMin: { width: 72 },
  idleMin: { width: 72 },
  status: { width: 104 },
  cmd: { grow: true },
  kill: { width: 72 },
};

const PROC_COLUMNS: readonly TableColumn[] = PROC_SCHEMA.filter((field) => field.path !== 'pid').map((field) => ({ path: field.path, ...COLUMN_SIZE[field.path] }));

export { CUSTOM_PATHS, PROC_COLUMNS, PROC_SCHEMA, STATUS_TIP, STATUS_TONE };
