/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { RecordLink, TypeIcon } from '../../../../../shell';
import type { AgentLineProps } from '../kanban-props.type';
import { AGENT_KIND } from '../kanban.constants';
import { agentTone } from '../model';

const AgentLine = (props: AgentLineProps) => {
  const { item } = props;
  const agents = item.agents ?? [];
  if (agents.length === 0) return null;
  return (
    <Flex gap="xs" align="center" wrap data-agent-line={item.id}>
      <TypeIcon kind={AGENT_KIND} />
      {agents.map((agent) => (
        <Flex key={agent.id} gap="xs" align="center">
          <RecordLink id={agent.id}>
            <Text as="span" variant="caption">{agent.name}</Text>
          </RecordLink>
          <Tag variant="urgency" color={agentTone(agent.status)}>{agent.status ?? 'working'}</Tag>
        </Flex>
      ))}
    </Flex>
  );
};

export { AgentLine };
