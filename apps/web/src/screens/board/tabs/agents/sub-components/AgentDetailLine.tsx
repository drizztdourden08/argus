/* @layer renderer-app @kind component */
import { DateText, Flex, Text } from '@drizztdourden08/tessera';
import type { AgentDetailLineProps } from '../AgentsTab.type';
import { agentMessage, agentNow, clipText, MESSAGE_CLIP, plainText } from '../model';

const AgentDetailLine = (props: AgentDetailLineProps) => {
  const { agent } = props;
  const now = agentNow(agent);
  const message = plainText(agentMessage(agent));
  return (
    <Flex gap="sm" align="center">
      {now && (
        <Text variant="caption" tone="info">
          {now.text}
          {now.since && <> <DateText value={now.since} format="relative" /></>}
        </Text>
      )}
      {agent.note && <Text variant="caption" tone="secondary">{clipText(agent.note, MESSAGE_CLIP)}</Text>}
      {message && <Text variant="caption" tone="muted">{clipText(message, MESSAGE_CLIP)}</Text>}
    </Flex>
  );
};

export { AgentDetailLine };
