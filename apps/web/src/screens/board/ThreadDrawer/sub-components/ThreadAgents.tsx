/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../format';
import type { ThreadAgentsProps } from '../ThreadDrawer.type';
import { ThreadSection } from './ThreadSection';

const ThreadAgents = (props: ThreadAgentsProps) => {
  const { agents, now } = props;
  return (
    <ThreadSection title="Agents" count={agents.length}>
      <Box className="thread-agents">
        {agents.map((agent, index) => (
          <Box key={`${agent.name ?? 'agent'}-${index}`} className="thread-agents__row">
            <Text as="b" weight={600} className="thread-agents__name">{agent.name ?? 'agent'}</Text>
            <Text as="span">{agent.task ?? ''}</Text>
            <Text as="span" variant="caption">{agent.startedAt ? `${formatAgo(agent.startedAt, now)} ago` : ''}</Text>
          </Box>
        ))}
      </Box>
    </ThreadSection>
  );
};

export { ThreadAgents };
