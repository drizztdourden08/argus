/* @layer renderer-app @kind component */
import { Flex, Tag, Tooltip } from '@drizztdourden08/tessera';
import type { AgentTrailingProps } from '../row-props.type';
import { agentMarks, isActive } from '../model';
import { AgentNow } from './AgentNow';
import { AgentStatusTag } from './AgentStatusTag';
import { AgentStopButton } from './AgentStopButton';

const AgentTrailing = (props: AgentTrailingProps) => {
  const { id, agent, now, actions } = props;
  const marks = agentMarks(agent, now).filter((mark) => mark.id === 'stale');
  return (
    <Flex gap="xs" align="center">
      {marks.map((mark) => (
        <Tooltip key={mark.id} content={mark.tip}>
          <Tag variant="urgency" color={mark.tone}>{mark.label}</Tag>
        </Tooltip>
      ))}
      {agent.status !== 'working' && <AgentStatusTag agent={agent} />}
      {isActive(agent) && agent.live && <AgentNow agent={agent} now={now} />}
      <AgentStopButton id={id} agent={agent} actions={actions} />
    </Flex>
  );
};

export { AgentTrailing };
