/* @layer renderer-app @kind component */
import { Box, Button, CopyButton, Flex, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import { BoardMarkdown } from '../../../../../shell';
import type { AgentFullMessageProps } from '../AgentsTab.type';
import { agentMessage, agentName } from '../model';

const AgentFullMessage = (props: AgentFullMessageProps) => {
  const { id, agent, now, onClose } = props;
  const message = agentMessage(agent);
  const at = agent.live?.lastActivityAt;
  return (
    <Box role="row" className="agents-expand" data-agent-message={id}>
      <Box role="cell">
        <Text as="p" variant="caption">{`Last message of ${agentName(agent, id)}${at ? ` · ${formatAgo(at, now)} ago` : ''}`}</Text>
        <BoardMarkdown>{message}</BoardMarkdown>
      </Box>
      <Flex gap="xs" align="start">
        <CopyButton text={message} label="Copy the message" showLabel size="xs" variant="ghost" />
        <Button size="xs" variant="ghost" onClick={() => onClose(id)}>Close</Button>
      </Flex>
    </Box>
  );
};

export { AgentFullMessage };
