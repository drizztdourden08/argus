/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { jobText, leftoverFlags, STATUS_LABEL, STATUS_LOOK, statusTip, UNKNOWN_STATUS_LOOK, WARNING_LOOK } from '../model';
import type { CellProps } from '../worktrees.type';

const StatusCell = (props: CellProps) => {
  const { row } = props;
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="xs" align="center" wrap>
        <Tag {...(STATUS_LOOK[row.status] ?? UNKNOWN_STATUS_LOOK)} title={statusTip(row)}>{STATUS_LABEL[row.status] ?? row.status}</Tag>
        {row.main && <Tag title="The repository's own checkout, not a linked worktree">main</Tag>}
        {row.locked && <Tag title={`Locked: ${String(row.locked)}`}>locked</Tag>}
        {row.detached && <Tag title="HEAD is detached">detached</Tag>}
        {leftoverFlags(row).map((flag) => <Tag key={flag.text} {...WARNING_LOOK} title={flag.tip}>{flag.text}</Tag>)}
      </Flex>
      {row.job && <Text variant="caption" tone={row.job.state === 'failed' ? 'danger' : 'muted'}>{jobText(row.job)}</Text>}
    </Flex>
  );
};

export { StatusCell };
