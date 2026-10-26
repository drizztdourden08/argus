/* @layer renderer-app @kind component */
import { CopyButton, Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import type { AgentDrawerBodyProps } from '../AgentsTab.type';
import { agentMessage, isActive, orText } from '../model';
import { AgentFacts } from './AgentFacts';
import { AgentModel } from './AgentModel';
import { AgentStatus } from './AgentStatus';
import { DrawerSection } from './DrawerSection';

const AgentDrawerBody = (props: AgentDrawerBodyProps) => {
  const { agent, now } = props;
  const message = agentMessage(agent);
  const noResult = isActive(agent) ? 'Not finished yet.' : 'No result recorded.';
  return (
    <Flex direction="column" gap="md">
      <Flex gap="sm" align="center">
        <AgentStatus agent={agent} />
        <AgentModel agent={agent} />
      </Flex>
      <AgentFacts agent={agent} now={now} />
      <DrawerSection title="Task"><Text>{orText(agent.task, 'No task recorded.')}</Text></DrawerSection>
      <DrawerSection title="Note"><Text>{orText(agent.note, 'No note.')}</Text></DrawerSection>
      <DrawerSection title="Result">
        {agent.result ? <BoardMarkdown>{agent.result}</BoardMarkdown> : <Text tone="muted">{noResult}</Text>}
      </DrawerSection>
      {message && (
        <DrawerSection title="Last message">
          <BoardMarkdown>{message}</BoardMarkdown>
          <CopyButton text={message} label="Copy the message" showLabel size="sm" />
        </DrawerSection>
      )}
    </Flex>
  );
};

export { AgentDrawerBody };
