/* @layer renderer-app @kind component */
import { Grid } from '@drizztdourden08/tessera';
import { AGENT_CARD_WIDTH } from '../model';
import type { AgentCellsProps } from '../row-props.type';
import { AgentCard } from './AgentCard';
import { AgentRow } from './AgentRow';

const AgentCells = (props: AgentCellsProps) => {
  const { agents, layout, ...row } = props;
  if (layout === 'list') return <>{agents.map((agent) => <AgentRow key={agent.id} id={agent.id} agent={agent} layout={layout} {...row} />)}</>;
  const { boardKey, threads, now, actions, onOpen } = row;
  return (
    <Grid minColWidth={AGENT_CARD_WIDTH} gap="sm" className="agents-grid">
      {agents.map((agent) => <AgentCard key={agent.id} boardKey={boardKey} id={agent.id} agent={agent} threads={threads} now={now} actions={actions} onOpen={onOpen} />)}
    </Grid>
  );
};

export { AgentCells };
