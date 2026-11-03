/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../kanban/sub-components/DrawerSection';
import type { AgentSessionProps } from '../row-props.type';
import { tokenLines } from '../model';

const AgentSession = (props: AgentSessionProps) => {
  const { agent } = props;
  return (
    <DrawerSection title="Session">
      <Flex direction="column" gap="2xs">
        {tokenLines(agent).map((line) => <Text key={line} variant="caption">{line}</Text>)}
        {agent.claudeId && <Text variant="caption" mono>{`Session id ${agent.claudeId}`}</Text>}
      </Flex>
    </DrawerSection>
  );
};

export { AgentSession };
