/* @layer renderer-app @kind logic */
import type { CacheRow } from '../../../../../data/cache';
import { KIND_LABEL } from './model.constants';
import type { CacheTableRow } from './model.type';
import { usersText } from './users-text';

const toCacheRow = (row: CacheRow): CacheTableRow => ({
  id: `${row.kind}/${row.name}@${row.version}`,
  kind: KIND_LABEL[row.kind] ?? row.kind,
  name: row.name,
  version: row.version,
  builds: row.buildKeys,
  users: usersText(row.users),
  userCount: row.userCount,
  sizeBytes: row.sizeBytes,
  lastUsedAt: row.lastUsedAt,
  item: row,
});

export { toCacheRow };
