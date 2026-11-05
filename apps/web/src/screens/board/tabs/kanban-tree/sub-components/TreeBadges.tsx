/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { StatusTag } from '../../kanban/sub-components/StatusTag';
import { progressOf } from '../model';
import type { TreeRowProps } from '../tree.type';

const TreeBadges = (props: TreeRowProps) => {
  const { node } = props;
  const progress = progressOf(node);
  return (
    <Flex gap="xs" align="center">
      {node.kind === 'case' ? <StatusTag status={node.status} /> : <Tag>{node.stage}</Tag>}
      {progress && <Text as="span" variant="caption" tone="muted" numeric title={progress.title}>{progress.text}</Text>}
    </Flex>
  );
};

export { TreeBadges };
