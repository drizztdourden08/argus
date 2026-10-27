/* @layer renderer-app @kind component */
import type { AgentRowProps } from '../AgentsTab.type';
import { agentMessage } from '../model';
import { AgentFullMessage } from './AgentFullMessage';
import { AgentNoteEdit } from './AgentNoteEdit';
import { AgentRowMain } from './AgentRowMain';

const AgentRow = (props: AgentRowProps) => {
  const { id, agent, now, actions, state } = props;
  return (
    <>
      <AgentRowMain {...props} />
      {state.openMessages.has(id) && agentMessage(agent) !== '' && <AgentFullMessage id={id} agent={agent} now={now} onClose={state.toggleMessage} />}
      {state.editing === id && <AgentNoteEdit id={id} agent={agent} actions={actions} onDone={state.stopEdit} />}
    </>
  );
};

export { AgentRow };
