/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { formatBytes } from '../../../../format';
import { Age } from '../../../../shell';
import type { BuildCellProps } from '../registry.type';

const BuildCell = (props: BuildCellProps) => {
  const { row, path } = props;
  if (path === 'name') return <Text size="md" weight={600}>{row.name}</Text>;
  if (path === 'version') return <Text size="md" numeric>{row.version}</Text>;
  if (path === 'commit') return <Text size="md" tone="dim" mono>{row.commit}</Text>;
  if (path === 'checkout') return <Text size="md" tone="dim" title={row.checkout}>{row.checkout}</Text>;
  if (path === 'size') return <Text size="md" tone="dim" numeric>{formatBytes(row.size)}</Text>;
  return <Text size="md" tone="dim" numeric><Age value={row.packedAt} /></Text>;
};

export { BuildCell };
