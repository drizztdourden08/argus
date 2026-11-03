/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { CellProps } from '../worktrees.type';

const RowFlagTags = (props: CellProps) => {
  const { row } = props;
  return (
    <>
      {row.main && <Tag title="The repository's own checkout, not a linked worktree">main</Tag>}
      {row.locked && <Tag title={`Locked: ${String(row.locked)}`}>locked</Tag>}
      {row.detached && <Tag title="HEAD is detached">detached</Tag>}
    </>
  );
};

export { RowFlagTags };
