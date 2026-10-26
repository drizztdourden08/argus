/* @layer renderer-app @kind component */
import { DateText, Flex, Text } from '@drizztdourden08/tessera';
import type { ThreadAgentsProps } from '../ThreadDrawer.type';
import { ThreadSection } from './ThreadSection';

const ThreadAgents = (props: ThreadAgentsProps) => {
  const { agents } = props;
  return (
    <ThreadSection title="Agents" count={agents.length}>
      <Flex direction="column" gap="xs">
        {agents.map((agent, index) => (
          <Flex key={`${agent.name ?? 'agent'}-${index}`} gap="sm" align="baseline" wrap>
            <Text variant="label">{agent.name ?? 'agent'}</Text>
            <Text>{agent.task ?? ''}</Text>
            {agent.startedAt ? <DateText value={agent.startedAt} format="relative" /> : null}
          </Flex>
        ))}
      </Flex>
    </ThreadSection>
  );
};

export { ThreadAgents };
