/* @layer renderer-app @kind component */
import { Flex, Pressable, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import type { AgentNameProps } from '../AgentsTab.type';
import { agentMarks, agentMessage, agentName, isActive, plainText } from '../model';
import { AgentCell } from './AgentCell';
import { AgentNow } from './AgentNow';
import { AgentRunChips } from './AgentRunChips';

const AgentName = (props: AgentNameProps) => {
  const { id, agent, now, runs, onOpen, onLog } = props;
  const name = agentName(agent, id);
  const message = plainText(agentMessage(agent));
  const marks = agentMarks(agent, now).filter((mark) => mark.id === 'stale');
  return (
    <AgentCell name="agent">
      <Flex className="agents-name" align="center" gap="xs">
        <Pressable
          className="agents-name__link"
          title={agent.task ? `${name}\n${agent.task}` : name}
          onClick={(event) => {
            event.stopPropagation();
            onOpen(id);
          }}
        >
          {name}
        </Pressable>
        <AgentRunChips runs={runs} onLog={onLog} />
        {marks.map((mark) => (
          <Tooltip key={mark.id} content={mark.tip}>
            <Tag variant="urgency" color={mark.tone}>{mark.label}</Tag>
          </Tooltip>
        ))}
      </Flex>
      {isActive(agent) && agent.live && <Text as="span" className="agents-name__now"><AgentNow agent={agent} now={now} /></Text>}
      {message && <Text as="span" className="agents-name__message">{message}</Text>}
    </AgentCell>
  );
};

export { AgentName };
