/* @layer renderer-app @kind component */
import { Flex, Icon, Tag, Text } from '@drizztdourden08/tessera';
import { shortCommit } from '../../../../../format';
import { Age, ReplayTag } from '../../../../../shell';
import type { PrPartProps } from '../changes.type';

const PrMeta = (props: PrPartProps) => {
  const { pr } = props;
  return (
    <Flex gap="md" align="center" wrap>
      <Text variant="caption" tone="muted" title="Branch and base">
        <Text variant="caption" tone="dim" mono>{pr.head}</Text>
        <Icon name="arrow-right" size={12} aria-label="into" />
        <Text variant="caption" tone="dim" mono>{pr.base}</Text>
      </Text>
      {pr.item && <Tag pill icon="link" href={`#${pr.item}`} title={`Open item ${pr.item} on the Kanban tab`}>{pr.item}</Tag>}
      {pr.createdBy && <Text variant="caption" tone="muted">{`by ${pr.createdBy}`}</Text>}
      {pr.createdAt && <Text variant="caption" tone="muted">{'opened '}<Age value={pr.createdAt} ago /></Text>}
      {pr.status === 'merged' && pr.mergedAt && <Text variant="caption" tone="muted">{'merged '}<Age value={pr.mergedAt} ago /></Text>}
      {pr.mergeCommit && <Text variant="caption" tone="muted" title="Merge commit">{'commit '}<Text variant="caption" tone="dim" mono>{shortCommit(pr.mergeCommit)}</Text></Text>}
      <ReplayTag replay={pr.replay} />
    </Flex>
  );
};

export { PrMeta };
