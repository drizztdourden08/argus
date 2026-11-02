/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { hygieneTag, jobText, leftoverFlags, STATUS_LABEL, STATUS_LOOK, statusShown, statusTip, UNKNOWN_STATUS_LOOK, WARNING_LOOK } from '../model';
import type { CellProps } from '../worktrees.type';
import { RowFlagTags } from './RowFlagTags';

const StatusCell = (props: CellProps) => {
  const { row } = props;
  const hygiene = row.hygiene ? hygieneTag(row.hygiene) : null;
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="xs" align="center" wrap>
        {statusShown(row) && <Tag {...(STATUS_LOOK[row.status] ?? UNKNOWN_STATUS_LOOK)} title={statusTip(row)}>{STATUS_LABEL[row.status] ?? row.status}</Tag>}
        <RowFlagTags {...props} />
        {hygiene && <Tag {...hygiene.look} fill="tint" title={hygiene.tip}>{hygiene.text}</Tag>}
        {leftoverFlags(row).map((flag) => <Tag key={flag.text} {...WARNING_LOOK} title={flag.tip}>{flag.text}</Tag>)}
      </Flex>
      {row.job && <Text variant="caption" tone={row.job.state === 'failed' ? 'danger' : 'muted'}>{jobText(row.job)}</Text>}
    </Flex>
  );
};

export { StatusCell };
