/* @layer renderer-app @kind component */
import { Flex, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import type { AgentEyebrowProps } from '../drawer-props.type';
import { agentMarks } from '../model';
import { AgentModelTag } from './AgentModelTag';
import { AgentStatusTag } from './AgentStatusTag';

const AgentEyebrow = (props: AgentEyebrowProps) => {
  const { id, agent, now } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Text as="span" variant="caption" weight={600} mono>{id}</Text>
      <AgentStatusTag agent={agent} />
      <AgentModelTag agent={agent} />
      {agent.kind && <Text as="span" variant="caption">{agent.kind}</Text>}
      {agentMarks(agent, now).map((mark) => (
        <Tooltip key={mark.id} content={mark.tip}>
          <Tag variant="urgency" color={mark.tone}>{mark.label}</Tag>
        </Tooltip>
      ))}
    </Flex>
  );
};

export { AgentEyebrow };
