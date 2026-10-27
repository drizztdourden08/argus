/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { AgentNoteProps } from '../AgentsTab.type';
import { agentNote, isActive } from '../model';
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
      <Text as="span" tone={note ? undefined : 'muted'}>{note || '-'}</Text>
    </AgentCell>
  );
};

export { AgentNote };
