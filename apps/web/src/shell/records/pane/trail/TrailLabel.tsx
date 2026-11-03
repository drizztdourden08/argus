/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { edgeText } from '../model/edge-text';
import { nodeRef } from '../model/node-ref';
import type { TrailLabelProps } from './trail.type';
import { StatusTag } from '../../../../screens/board/tabs/kanban/sub-components/StatusTag';

const TrailLabel = (props: TrailLabelProps) => {
  const { node, boardKey, now } = props;
  return (
    <Flex gap="xs" align="center" wrap>
      <Text as="span" variant="caption" weight={600} mono>{nodeRef(node, boardKey)}</Text>
      <Text as="span" smartTimes>{node.title}</Text>
      {node.lane && <StatusTag status={node.lane} />}
      {node.stage && node.stage !== node.lane && <Tag>{node.stage}</Tag>}
      {node.assignee && <Text as="span" variant="caption" tone="muted">{node.assignee}</Text>}
      {node.cycle && <Tag variant="urgency" color="warning">cycle</Tag>}
      {node.edge && <Text as="span" variant="caption" tone="muted">{edgeText(node.edge, now)}</Text>}
    </Flex>
  );
};

export { TrailLabel };
