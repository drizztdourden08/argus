/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { AgentDrawerHeadProps } from '../drawer-props.type';
import { AgentItemSelect } from './AgentItemSelect';
import { AgentStopLine } from './AgentStopLine';

const revLine = (props: AgentDrawerHeadProps): string => {
  const { agent, now } = props;
  const by = agent.updatedBy ?? agent.createdBy ?? 'unknown';
  return `rev ${agent.rev ?? 1}, updated by ${by} ${formatAgo(agent.updatedAt, now)} ago`;
};

const AgentDrawerHead = (props: AgentDrawerHeadProps) => {
  const { boardKey, id, agent, actions } = props;
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="md" align="center" wrap>
        <AgentItemSelect boardKey={boardKey} id={id} agent={agent} actions={actions} />
        <Text as="span" variant="caption">{`Thread ${agent.thread ?? ''}`}</Text>
        <AgentStopLine id={id} agent={agent} actions={actions} />
      </Flex>
      <Text variant="caption">
        {revLine(props)}
        {agent.claudeId && ` · Claude id ${agent.claudeId}`}
      </Text>
    </Flex>
  );
};

export { AgentDrawerHead };
