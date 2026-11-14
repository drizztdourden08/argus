/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { agentTone, itemAgents } from '../../../screens/board/tabs/kanban/model';
import { RecordLink } from '../sub-components/RecordLink';
import type { PaneBodyProps } from './pane.type';

const ItemAgents = (props: PaneBodyProps) => {
  const { item, agents } = props.data;
  const linked = itemAgents(agents, item.id);
  if (linked.length === 0) return null;
  return (
    <Flex direction="column" gap="xs" data-item-agents={item.id}>
      <Text variant="overline" tone="muted">{`Agents · ${linked.length}`}</Text>
      {linked.map((agent) => (
        <Flex key={agent.id} gap="sm" align="center" wrap>
          <RecordLink id={agent.id}>
            <Text as="span" weight={600}>{agent.name?.trim() || agent.id}</Text>
          </RecordLink>
          <Text as="span" variant="caption" tone="muted">{agent.model ?? 'model not set'}</Text>
          <Tag variant="urgency" color={agentTone(agent.status)}>{agent.status ?? 'working'}</Tag>
        </Flex>
      ))}
    </Flex>
  );
};

export { ItemAgents };
