/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import type { NameCellProps } from '../processes.type';

const NameCell = (props: NameCellProps) => {
  const { name, pid } = props;
  return (
    <Inline gap="xs" align="baseline">
      <Text size="md" weight={600}>{name}</Text>
      {pid === null ? null : <Text size="md" tone="muted" numeric>{pid}</Text>}
    </Inline>
  );
};

export { NameCell };
