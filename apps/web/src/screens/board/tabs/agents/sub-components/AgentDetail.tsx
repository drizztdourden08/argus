/* @layer renderer-app @kind component */
import type { AgentDetailProps } from '../row-props.type';
import { hasMessage } from '../model';
import { AgentFullMessage } from './AgentFullMessage';
import { AgentNoteEdit } from './AgentNoteEdit';

const AgentDetail = (props: AgentDetailProps) => {
  const { id, agent, now, actions, state } = props;
  return (
    <>
      {hasMessage(agent, id, state) && <AgentFullMessage id={id} agent={agent} now={now} onClose={state.toggleMessage} />}
      {state.editing === id && <AgentNoteEdit id={id} agent={agent} actions={actions} onDone={state.stopEdit} />}
    </>
  );
};

export { AgentDetail };
