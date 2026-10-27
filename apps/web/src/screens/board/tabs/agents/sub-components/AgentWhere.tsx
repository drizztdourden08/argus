/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { AgentFieldProps } from '../drawer-props.type';
import { orText } from '../model';
import { AgentWhereField } from './AgentWhereField';

const AgentWhere = (props: AgentFieldProps) => {
  const { id, agent, actions } = props;
  return (
    <Box as="dl" className="agents-kv">
      <Box as="dt" className="agents-kv__label">Model</Box>
      <Box as="dd">{orText(agent.model, '-')}</Box>
      <Box as="dt" className="agents-kv__label">Kind</Box>
      <Box as="dd">{orText(agent.kind, '-')}</Box>
      <Box as="dt" className="agents-kv__label">Branch</Box>
      <Box as="dd"><AgentWhereField id={id} agent={agent} actions={actions} field="branch" /></Box>
      <Box as="dt" className="agents-kv__label">Worktree</Box>
      <Box as="dd"><AgentWhereField id={id} agent={agent} actions={actions} field="worktree" /></Box>
    </Box>
  );
};

export { AgentWhere };
