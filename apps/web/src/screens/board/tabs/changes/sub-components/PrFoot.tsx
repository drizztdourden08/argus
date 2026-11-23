/* @layer renderer-app @kind component */
import { Flex, Status, Tag, Text } from '@drizztdourden08/tessera';
import { ReplayTag } from '../../../../../shell';
import { PR_STATUSES } from '../changes.constants';
import type { PrFootProps } from '../changes.type';
import { checksProgress } from '../model';

const PrFoot = (props: PrFootProps) => {
  const { pr, runs, compact } = props;
  const progress = checksProgress(runs);
  return (
    <Flex gap="sm" align="center" wrap>
      {!compact && <Tag title="Branch into base">{`${pr.head} into ${pr.base}`}</Tag>}
      <Status map={PR_STATUSES} value={pr.status} dot size="sm" />
      {progress && <Text variant="caption" tone={progress.failed ? 'danger' : 'muted'} title="Steps passed in the newest check run">{`checks ${progress.passed}/${progress.total}`}</Text>}
      {!compact && <ReplayTag replay={pr.replay} />}
    </Flex>
  );
};

export { PrFoot };
