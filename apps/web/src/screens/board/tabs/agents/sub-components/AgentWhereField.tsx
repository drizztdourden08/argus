/* @layer renderer-app @kind component */
import { CopyButton, Flex, InlineEdit } from '@drizztdourden08/tessera';
import type { AgentWhereFieldProps } from '../drawer-props.type';
import { agentProblem } from '../model';

const AgentWhereField = (props: AgentWhereFieldProps) => {
  const { id, agent, actions, field } = props;
  const value = agent[field] ?? '';
  return (
    <Flex gap="xs" align="center">
      <InlineEdit
        label={field}
        placeholder={`add ${field}`}
        value={value}
        validate={(text) => agentProblem(field, text) ?? undefined}
        onCommit={(next) => next.trim() !== value && actions.setField(id, field, next.trim())}
      />
      {value && <CopyButton text={value} label={`Copy the ${field}`} variant="ghost" size="xs" />}
    </Flex>
  );
};

export { AgentWhereField };
