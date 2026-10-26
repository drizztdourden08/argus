/* @layer renderer-app @kind component */
import { Flex, Status, Tag, Text } from '@drizztdourden08/tessera';
import { isKnownStatus, jobText, leftoverFlags, statusTip, WORKTREE_STATUSES } from '../model';
import type { CellProps } from '../worktrees.type';

const StatusCell = (props: CellProps) => {
  const { row } = props;
  const tip = statusTip(row);
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="xs" align="center" wrap>
        {isKnownStatus(row.status)
          ? <Status map={WORKTREE_STATUSES} value={row.status} variant="pill" title={tip} />
          : <Status tone="neutral" variant="pill" title={tip}>{row.status}</Status>}
        {row.main && <Tag title="The repository's own checkout, not a linked worktree">main</Tag>}
        {row.locked && <Tag title={`Locked: ${String(row.locked)}`}>locked</Tag>}
        {row.detached && <Tag title="HEAD is detached">detached</Tag>}
        {leftoverFlags(row).map((flag) => <Tag key={flag.text} title={flag.tip}>{flag.text}</Tag>)}
      </Flex>
      {row.job && <Text variant="caption" tone={row.job.state === 'failed' ? 'danger' : 'muted'}>{jobText(row.job)}</Text>}
    </Flex>
  );
};

export { StatusCell };
