/* @layer renderer-app @kind component */
import { Flex, Text, Tooltip } from '@drizztdourden08/tessera';
import { cellSize, filesText } from '../model';
import type { CellProps } from '../worktrees.type';

const LeftoverSize = (props: CellProps) => {
  const { row } = props;
  const files = filesText(row);
  const tops = (row.leftover?.topEntries ?? []).join(', ');
  const size = <Text>{cellSize(row)}</Text>;
  return (
    <Flex direction="column" gap="2xs">
      {tops ? <Tooltip content={tops}>{size}</Tooltip> : size}
      {files && <Text variant="caption" tone="muted">{files}</Text>}
    </Flex>
  );
};

export { LeftoverSize };
