/* @layer renderer-app @kind component */
import { Flex, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { AgentDrawerHeadProps } from '../drawer-props.type';
import { agentMarks } from '../model';
import { AgentItemSelect } from './AgentItemSelect';
import { AgentModelTag } from './AgentModelTag';
import { AgentStatusTag } from './AgentStatusTag';
import { AgentStopLine } from './AgentStopLine';

const revLine = (props: AgentDrawerHeadProps): string => {
  const { agent, now } = props;
  const by = agent.updatedBy ?? agent.createdBy ?? 'unknown';
  return `rev ${agent.rev ?? 1}, updated by ${by} ${formatAgo(agent.updatedAt, now)} ago`;
};

const AgentDrawerHead = (props: AgentDrawerHeadProps) => {
  const { boardKey, id, agent, now, actions } = props;
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="sm" align="center" wrap>
        <AgentStatusTag agent={agent} />
        <AgentModelTag agent={agent} />
        {agent.kind && <Text as="span" variant="caption">{agent.kind}</Text>}
        {agentMarks(agent, now).map((mark) => (
          <Tooltip key={mark.id} content={mark.tip}>
            <Tag variant="urgency" color={mark.tone}>{mark.label}</Tag>
          </Tooltip>
        ))}
      </Flex>
      <Flex gap="md" align="center" wrap>
        <AgentItemSelect boardKey={boardKey} id={id} agent={agent} actions={actions} />
        <Text as="span" variant="caption">{`Thread ${agent.thread ?? ''}`}</Text>
        <AgentStopLine id={id} agent={agent} actions={actions} />
      </Flex>
      <Text variant="caption">
        {revLine(props)}
        {agent.claudeId && ` · Agent id ${agent.claudeId}`}
      </Text>
    </Flex>
  );
};

export { AgentDrawerHead };
