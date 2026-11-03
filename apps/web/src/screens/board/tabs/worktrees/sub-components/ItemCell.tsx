/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import type { CellProps } from '../worktrees.type';

const ItemCell = (props: CellProps) => {
  const { row } = props;
  return (
    <Flex gap="xs" align="center" wrap>
      {row.item && <Tag pill icon="link" href={`#${row.item}`} title={`Open ${row.item} on the Kanban tab${row.linkSource ? ` (linked by ${row.linkSource})` : ''}`}>{row.item}</Tag>}
      {!row.item && row.unlinked && <Tag variant="urgency" color="warning" title="No item an hour after Argus first saw it: link it (coord worktree link) or remove it. It is never removed for this alone.">no item</Tag>}
    </Flex>
  );
};

export { ItemCell };
