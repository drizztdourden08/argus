/* @layer renderer-app @kind component */
import { Flex, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import type { RecordBodyProps } from '../../../../../shell/records/pane/pane.type';
import { DrawerSection } from '../../kanban/sub-components/DrawerSection';
import { useAgentActions } from '../behavior/useAgentActions';
import { useAgentDoc } from '../behavior/useAgentDoc';
import { AgentModelTag } from '../sub-components/AgentModelTag';
import { AgentStatusTag } from '../sub-components/AgentStatusTag';
import { AgentTextField } from '../sub-components/AgentTextField';
import { AgentTimeline } from '../sub-components/AgentTimeline';
import { agentMarks, agentMessage, agentNow, isActive, tokenLines } from '../model';

const AgentTranscriptBody = (props: RecordBodyProps) => {
  const { boardKey, target, now } = props.data;
  const { agent, agents, ready } = useAgentDoc(boardKey, target.id);
  const actions = useAgentActions(boardKey, agents);
  if (!agent) return <Text tone="muted">{ready ? `${target.id} is not on this board.` : 'Loading the agent.'}</Text>;
  const doing = agentNow(agent);
  const message = agentMessage(agent);
  const noResult = isActive(agent) ? 'Not finished yet.' : 'No result recorded.';
  return (
    <Flex direction="column" gap="lg">
      <Flex gap="sm" align="center" wrap>
        <AgentStatusTag agent={agent} />
        <AgentModelTag agent={agent} />
        {agentMarks(agent, now).map((mark) => (
          <Tooltip key={mark.id} content={mark.tip}>
            <Tag variant="urgency" color={mark.tone}>{mark.label}</Tag>
          </Tooltip>
        ))}
      </Flex>
      <DrawerSection title="Task"><AgentTextField id={agent.id} agent={agent} actions={actions} field="task" /></DrawerSection>
      <DrawerSection title="Note"><AgentTextField id={agent.id} agent={agent} actions={actions} field="note" /></DrawerSection>
      <DrawerSection title="Now">
        <Text tone={doing ? undefined : 'muted'}>{doing?.text ?? agent.live?.liveNote ?? 'Nothing running right now.'}</Text>
      </DrawerSection>
      <DrawerSection title="Last message">
        {message ? <BoardMarkdown>{message}</BoardMarkdown> : <Text tone="muted">No message yet.</Text>}
      </DrawerSection>
      <DrawerSection title="Result">
        {agent.result ? <BoardMarkdown>{agent.result}</BoardMarkdown> : <Text tone="muted">{noResult}</Text>}
      </DrawerSection>
      <DrawerSection title="Session">
        <Flex direction="column" gap="2xs">
          {tokenLines(agent).map((line) => <Text key={line} variant="caption">{line}</Text>)}
          {agent.claudeId && <Text variant="caption" mono>{`Claude id ${agent.claudeId}`}</Text>}
        </Flex>
      </DrawerSection>
      <DrawerSection title="Timeline"><AgentTimeline agent={agent} now={now} /></DrawerSection>
    </Flex>
  );
};

export { AgentTranscriptBody };
