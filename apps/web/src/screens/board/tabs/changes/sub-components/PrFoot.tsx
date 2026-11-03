/* @layer renderer-app @kind component */
import { Flex, Spacer, Status, Tag, Text } from '@drizztdourden08/tessera';
import { Age, ReplayTag } from '../../../../../shell';
import { PR_STATUSES } from '../changes.constants';
import type { PrFootProps } from '../changes.type';
import { checksProgress } from '../model';

const PrFoot = (props: PrFootProps) => {
  const { pr, runs } = props;
  const progress = checksProgress(runs);
  const stamp = pr.status === 'merged' && pr.mergedAt ? pr.mergedAt : pr.createdAt;
  return (
    <Flex gap="sm" align="center" wrap>
      <Tag title="Branch into base">{`${pr.head} into ${pr.base}`}</Tag>
      <Status map={PR_STATUSES} value={pr.status} dot size="sm" />
      {progress && <Text variant="caption" tone={progress.failed ? 'danger' : 'muted'} title="Steps passed in the newest check run">{`checks ${progress.passed}/${progress.total}`}</Text>}
      <ReplayTag replay={pr.replay} />
      <Spacer />
      {stamp ? <Text variant="caption" tone="muted"><Age value={stamp} /></Text> : null}
    </Flex>
  );
};

export { PrFoot };
