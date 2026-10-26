/* @layer renderer-app @kind constants */
import type { FieldDescriptor, TableColumn, TagLook } from '@drizztdourden08/tessera';

const STATUS_TIP: Readonly<Record<string, string>> = {
  ok: 'Within its rule.',
  hog: 'Uses more memory than its rule allows.',
  stale: 'Idle for longer than its rule allows.',
  orphan: 'Its parent process is gone.',
  hung: 'Not responding.',
  old: 'Running for longer than its rule allows.',
};

const WARNING_LOOK: TagLook = { variant: 'urgency', color: 'warning' };
const DANGER_LOOK: TagLook = { variant: 'urgency', color: 'danger' };

const STATUS_LOOK: Readonly<Record<string, TagLook>> = {
  ok: { variant: 'urgency', color: 'success' },
  hog: WARNING_LOOK,
  stale: WARNING_LOOK,
  old: WARNING_LOOK,
  orphan: DANGER_LOOK,
  hung: DANGER_LOOK,
};

const UNKNOWN_STATUS_LOOK: TagLook = { variant: 'normal', color: 'neutral' };

const STATUS_OPTIONS = ['ok', 'hog', 'stale', 'orphan', 'hung', 'old'] as const;

const PROC_SCHEMA: readonly FieldDescriptor[] = [
  { path: 'rule', label: 'Rule', kind: 'string', optional: false },
  { path: 'name', label: 'Process', kind: 'string', optional: false },
  { path: 'pid', label: 'PID', kind: 'number', optional: true },
  { path: 'project', label: 'Project', kind: 'string', optional: true },
  { path: 'memMB', label: 'Memory', kind: 'number', optional: true, unit: 'MB' },
  { path: 'cpuPct', label: 'CPU', kind: 'number', optional: true, unit: '%' },
  { path: 'ageMin', label: 'Age', kind: 'number', optional: true },
  { path: 'idleMin', label: 'Idle', kind: 'number', optional: true },
  { path: 'status', label: 'Status', kind: 'enum', optional: false, closed: true, options: STATUS_OPTIONS },
  { path: 'cmd', label: 'Command', kind: 'string', optional: true },
  { path: 'kill', label: 'Kill', kind: 'string', optional: true },
];

const CUSTOM_PATHS: readonly string[] = ['name', 'project', 'ageMin', 'idleMin', 'status', 'cmd', 'kill'];

const PROC_COLUMNS: readonly TableColumn[] = PROC_SCHEMA.filter((field) => field.path !== 'pid').map((field) => ({ path: field.path }));

export { CUSTOM_PATHS, PROC_COLUMNS, PROC_SCHEMA, STATUS_LOOK, STATUS_TIP, UNKNOWN_STATUS_LOOK };
