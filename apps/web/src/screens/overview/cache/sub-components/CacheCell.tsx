/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { formatBytes } from '../../../../format';
import { Age } from '../../../../shell';
import type { CacheCellProps } from '../cache.type';
import { UsersCell } from './UsersCell';

const CacheCell = (props: CacheCellProps) => {
  const { row, path } = props;
  if (path === 'kind') return <Text size="md" tone="dim">{row.kind}</Text>;
  if (path === 'name') return <Text size="md" weight={600}>{row.name}</Text>;
  if (path === 'version') return <Text size="md" numeric>{row.version}</Text>;
  if (path === 'users') return <UsersCell row={row} />;
  if (path === 'sizeBytes') return <Text size="md" tone="dim" numeric>{formatBytes(row.sizeBytes)}</Text>;
  return <Text size="md" tone="dim" numeric><Age value={row.lastUsedAt} /></Text>;
};

export { CacheCell };
