/* @layer renderer-app @kind component */
import { EditableMarkdown } from '../../../shared';
import type { AgentTextFieldProps } from '../drawer-props.type';
import { AGENT_LIMITS, agentProblem } from '../model';

const AgentTextField = (props: AgentTextFieldProps) => {
  const { id, agent, actions, field } = props;
  return (
    <EditableMarkdown
      noun={field}
      value={agent[field] ?? ''}
      empty={`No ${field}.`}
      limit={AGENT_LIMITS[field]}
      problem={(text) => agentProblem(field, text)}
      onSave={(value) => actions.setField(id, field, value)}
    />
  );
};

export { AgentTextField };
