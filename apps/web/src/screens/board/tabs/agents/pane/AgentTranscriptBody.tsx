/* @layer renderer-app @kind component */
import { Flex, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import type { RecordBodyProps } from '../../../../../shell/records/pane/pane.type';
import { DrawerSection } from '../../kanban/sub-components/DrawerSection';
import { useAgentActions } from '../behavior/useAgentActions';
import { useAgentDoc } from '../behavior/useAgentDoc';
import { AgentLiveNote } from '../sub-components/AgentLiveNote';
import { AgentModelTag } from '../sub-components/AgentModelTag';
import { AgentReply } from '../sub-components/AgentReply';
import { AgentSession } from '../sub-components/AgentSession';
import { AgentStatusTag } from '../sub-components/AgentStatusTag';
import { AgentTextField } from '../sub-components/AgentTextField';
import { AgentTimeline } from '../sub-components/AgentTimeline';
import { agentMarks, agentMessage, isActive } from '../model';

const AgentTranscriptBody = (props: RecordBodyProps) => {
  const { boardKey, target, now } = props.data;
  const { agent, agents, ready } = useAgentDoc(boardKey, target.id);
  const actions = useAgentActions(boardKey, agents);
  if (!agent) return <Text tone="muted">{ready ? `${target.id} is not on this board.` : 'Loading the agent.'}</Text>;
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
      <AgentLiveNote agent={agent} />
      <AgentReply title="Last message" text={agentMessage(agent)} empty="No message yet." />
      <AgentReply title="Result" text={agent.result} empty={isActive(agent) ? 'Not finished yet.' : 'No result recorded.'} />
      <AgentSession agent={agent} />
      <DrawerSection title="Timeline"><AgentTimeline agent={agent} now={now} /></DrawerSection>
    </Flex>
  );
};

export { AgentTranscriptBody };
