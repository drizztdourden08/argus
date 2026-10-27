/* @layer renderer-app @kind component */
import { Box, Pressable } from '@drizztdourden08/tessera';
import type { AgentMessageProps } from '../row-props.type';
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
      ) : <Box as="span" className="agents-na">-</Box>}
    </AgentCell>
  );
};

export { AgentMessage };
