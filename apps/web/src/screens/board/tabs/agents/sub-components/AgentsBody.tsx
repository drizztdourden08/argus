/* @layer renderer-app @kind component */
import { Box, Text, TrackTable } from '@drizztdourden08/tessera';
import type { AgentsBodyProps } from '../AgentsTab.type';
import { EMPTY_ALL, EMPTY_RUNNING, hasDetail } from '../model';
import { agentColumns } from './AgentColumns';
import { AgentDetail } from './AgentDetail';

const AgentsBody = (props: AgentsBodyProps) => {
  const { active, done, showFinished, onOpen, now, actions, state } = props;
  if (active.length + done.length === 0) return <Text as="p" tone="muted" className="agents-empty">{EMPTY_ALL}</Text>;
  const columns = agentColumns(props);
  const shared = {
    columns,
    rowKey: (row: { id: string }) => row.id,
    className: 'agents-table',
    onRowClick: (row: { id: string }) => onOpen(row.id),
    rowDetail: (row: (typeof active)[number]) => (hasDetail(row, row.id, state) ? <AgentDetail id={row.id} agent={row} now={now} actions={actions} state={state} /> : undefined),
  };
  return (
    <Box>
      <TrackTable {...shared} label="Sub-agents" rows={active} empty={EMPTY_RUNNING} />
      {showFinished && done.length > 0 && (
        <>
          <Text as="p" variant="overline" className="agents-group">Finished</Text>
          <TrackTable {...shared} label="Finished sub-agents" rows={done} head={false} />
        </>
      )}
    </Box>
  );
};

export { AgentsBody };
