/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import type { AgentMessageProps } from '../AgentsTab.type';
import { agentMessage, plainText } from '../model';
import { AgentCell } from './AgentCell';

const AgentMessage = (props: AgentMessageProps) => {
  const { id, agent, open, onToggle } = props;
  const message = agentMessage(agent);
  return (
    <AgentCell name="message">
      {message ? (
        <Pressable
          className="agents-message"
          aria-expanded={open}
          title={open ? 'Hide the full message' : 'Read the whole message'}
          onClick={(event) => {
            event.stopPropagation();
            onToggle(id);
          }}
        >
          {plainText(message)}
        </Pressable>
      ) : <Text as="span" tone="dim">-</Text>}
    </AgentCell>
  );
};

export { AgentMessage };
