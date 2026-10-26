/* @layer renderer-app @kind component */
import { Flex, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import type { AgentNameProps } from '../AgentsTab.type';
import { agentMarks, agentName, orText } from '../model';

const AgentName = (props: AgentNameProps) => {
  const { id, agent, now } = props;
  return (
    <Flex gap="xs" align="center">
      <Tooltip content={orText(agent.task, 'No task recorded')}>
        <Text weight={600}>{agentName(agent, id)}</Text>
      </Tooltip>
      <Text tone="muted" variant="caption">{id}</Text>
      {agentMarks(agent, now).map((mark) => (
        <Tooltip key={mark.id} content={mark.tip}>
          <Tag variant="urgency" color={mark.tone}>{mark.label}</Tag>
        </Tooltip>
      ))}
    </Flex>
  );
};

export { AgentName };
