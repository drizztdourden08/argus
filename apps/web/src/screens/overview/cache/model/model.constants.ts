/* @layer renderer-app @kind constants */
import type { FieldDescriptor, TableColumn } from '@drizztdourden08/tessera';

const KIND_LABEL: Readonly<Record<string, string>> = { package: 'Package', electron: 'Electron' };

const CACHE_SCHEMA: readonly FieldDescriptor[] = [
  { path: 'kind', label: 'Kind', kind: 'string', optional: false },
  { path: 'name', label: 'Name', kind: 'string', optional: false },
  { path: 'version', label: 'Version', kind: 'string', optional: false },
  { path: 'builds', label: 'Builds', kind: 'number', optional: false },
  { path: 'users', label: 'Used by', kind: 'string', optional: false },
  { path: 'userCount', label: 'Users', kind: 'number', optional: false },
  { path: 'sizeBytes', label: 'Size', kind: 'number', optional: false },
  { path: 'lastUsedAt', label: 'Last use', kind: 'string', optional: false },
];

const CUSTOM_PATHS: readonly string[] = ['kind', 'name', 'version', 'users', 'sizeBytes', 'lastUsedAt'];

const COLUMN_SIZE: Readonly<Record<string, Omit<TableColumn, 'path'>>> = {
  kind: { width: 90 },
  name: { width: 230 },
  version: { width: 100 },
  builds: { width: 72 },
  users: { grow: true },
  userCount: { width: 64 },
  sizeBytes: { width: 90 },
  lastUsedAt: { width: 90 },
};

const CACHE_COLUMNS: readonly TableColumn[] = CACHE_SCHEMA.map((field) => ({ path: field.path, ...COLUMN_SIZE[field.path] }));

export { CACHE_COLUMNS, CACHE_SCHEMA, CUSTOM_PATHS, KIND_LABEL };
