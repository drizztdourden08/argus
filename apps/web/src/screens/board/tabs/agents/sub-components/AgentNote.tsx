/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { AgentNoteProps } from '../row-props.type';
import { agentNote, isActive, orText } from '../model';
import { AgentCell } from './AgentCell';

const noteTip = (props: AgentNoteProps): string => {
  const { agent } = props;
  const note = agentNote(agent);
  const label = isActive(agent) || !agent.result ? 'Note' : 'Result';
  return `${agent.task ? `Task: ${agent.task}` : ''}${note ? `\n${label}: ${note}` : ''}`;
};

const AgentNote = (props: AgentNoteProps) => {
  const note = agentNote(props.agent);
  return (
    <AgentCell name="note" title={noteTip(props)}>
      <Box as="span" className={note ? undefined : 'agents-na'}>{orText(note, '-')}</Box>
    </AgentCell>
  );
};

export { AgentNote };
