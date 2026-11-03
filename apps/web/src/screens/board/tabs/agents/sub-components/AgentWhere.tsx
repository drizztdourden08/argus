/* @layer renderer-app @kind component */
import { Box, Checkbox, Tag, Tooltip } from '@drizztdourden08/tessera';
import type { AgentFieldProps } from '../drawer-props.type';
import { agentWhere, orText } from '../model';
import { AgentWhereField } from './AgentWhereField';

const AgentWhere = (props: AgentFieldProps) => {
  const { id, agent, actions } = props;
  const where = agentWhere(agent);
  return (
    <Box as="dl" className="agents-kv">
      <Box as="dt" className="agents-kv__label">Model</Box>
      <Box as="dd">{orText(agent.model, '-')}</Box>
      <Box as="dt" className="agents-kv__label">Kind</Box>
      <Box as="dd">{orText(agent.kind, '-')}</Box>
      <Box as="dt" className="agents-kv__label">Works in</Box>
      <Box as="dd">
        <Tooltip content={where.tip}>
          {where.kind === 'unknown' ? <Tag variant="urgency" color="warning">{where.text}</Tag> : <Tag color="neutral">{where.text}</Tag>}
        </Tooltip>
      </Box>
      <Box as="dt" className="agents-kv__label">Master</Box>
      <Box as="dd">
        <Checkbox checked={agent.master === true} onChange={() => actions.setMaster(id, agent.master !== true)} ariaLabel="Works on the master checkout" size="sm" />
      </Box>
      <Box as="dt" className="agents-kv__label">Branch</Box>
      <Box as="dd"><AgentWhereField id={id} agent={agent} actions={actions} field="branch" /></Box>
      <Box as="dt" className="agents-kv__label">Worktree</Box>
      <Box as="dd"><AgentWhereField id={id} agent={agent} actions={actions} field="worktree" /></Box>
    </Box>
  );
};

export { AgentWhere };
