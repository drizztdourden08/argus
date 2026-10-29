/* @layer renderer-app @kind constants */
import type { FieldDescriptor, TableColumn } from '@drizztdourden08/tessera';

const BUILD_SCHEMA: readonly FieldDescriptor[] = [
  { path: 'name', label: 'Package', kind: 'string', optional: false },
  { path: 'version', label: 'Version', kind: 'string', optional: false },
  { path: 'commit', label: 'Commit', kind: 'string', optional: false },
  { path: 'checkout', label: 'Checkout', kind: 'string', optional: false },
  { path: 'packedAt', label: 'Packed', kind: 'string', optional: false },
  { path: 'size', label: 'Size', kind: 'number', optional: false },
];

const COLUMN_SIZE: Readonly<Record<string, Omit<TableColumn, 'path'>>> = {
  name: { width: 260 },
  version: { width: 100 },
  commit: { width: 90 },
  checkout: { grow: true },
  packedAt: { width: 90 },
  size: { width: 90 },
};

const BUILD_COLUMNS: readonly TableColumn[] = BUILD_SCHEMA.map((field) => ({ path: field.path, ...COLUMN_SIZE[field.path] }));

export { BUILD_COLUMNS, BUILD_SCHEMA };
