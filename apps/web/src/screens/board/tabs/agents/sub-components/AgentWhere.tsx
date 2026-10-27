/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { AgentFieldProps } from '../AgentsTab.type';
import { AgentWhereField } from './AgentWhereField';

const AgentWhere = (props: AgentFieldProps) => {
  const { id, agent, actions } = props;
  return (
    <Box as="dl" className="agents-kv">
      <Text as="dt" variant="overline">Model</Text>
      <Text as="dd">{agent.model || '-'}</Text>
      <Text as="dt" variant="overline">Kind</Text>
      <Text as="dd">{agent.kind || '-'}</Text>
      <Text as="dt" variant="overline">Branch</Text>
      <Box as="dd"><AgentWhereField id={id} agent={agent} actions={actions} field="branch" /></Box>
      <Text as="dt" variant="overline">Worktree</Text>
      <Box as="dd"><AgentWhereField id={id} agent={agent} actions={actions} field="worktree" /></Box>
    </Box>
  );
};

export { AgentWhere };
