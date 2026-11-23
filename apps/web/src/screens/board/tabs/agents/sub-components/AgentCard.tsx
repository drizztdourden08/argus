/* @layer renderer-app @kind component */
import { Flex, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import { RecordCard, recordMenuGroups, ShownCode } from '../../../../../shell';
import { agentHandlers } from '../behavior/agent-handlers';
import { agentChips, agentItems, agentMarks, agentName, isActive } from '../model';
import type { AgentCardProps } from '../row-props.type';
import { AgentNow } from './AgentNow';
import { AgentStatusTag } from './AgentStatusTag';
import { AgentStopButton } from './AgentStopButton';

const AgentCard = (props: AgentCardProps) => {
  const { boardKey, id, agent, threads, now, actions, onOpen } = props;
  const name = agentName(agent, id);
  const stale = agentMarks(agent, now).filter((mark) => mark.id === 'stale');
  return (
    <RecordCard
      head={{
        kind: 'agent',
        code: id,
        assignee: agent.thread ?? '',
        boardKey,
        threads,
        menu: recordMenuGroups('agent', id, agentHandlers(id, agent, actions, onOpen)),
        menuLabel: `More actions for ${id}`,
      }}
      title={name}
      clamp={1}
      openLabel={`Open ${id}: ${name}`}
      onOpen={() => onOpen(id)}
      attrs={{ 'data-swap-item': '', 'data-record': id }}
      signals={(
        <Flex direction="column" gap="xs">
          {agent.task && <Text variant="caption" tone="dim" clamp={2} title={agent.task}>{agent.task}</Text>}
          <Flex gap="xs" wrap align="center">
            <AgentStatusTag agent={agent} />
            {stale.map((mark) => <Tooltip key={mark.id} content={mark.tip}><Tag variant="urgency" color={mark.tone}>{mark.label}</Tag></Tooltip>)}
            {agentChips(agent, now).map((chip) => <Tag key={chip.id} variant="normal" color="neutral" title={chip.title}>{chip.text}</Tag>)}
            {agentItems(agent).map((item) => <Tag key={item} variant="normal" color="neutral"><ShownCode id={item} board={boardKey} /></Tag>)}
          </Flex>
        </Flex>
      )}
      foot={(
        <Flex gap="xs" align="center" className="agents-card__foot">
          {isActive(agent) && agent.live && <AgentNow agent={agent} now={now} />}
          <AgentStopButton id={id} agent={agent} actions={actions} />
        </Flex>
      )}
    />
  );
};

export { AgentCard };
