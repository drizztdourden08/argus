/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { AgentRunsProps } from '../row-props.type';
import { AgentRunRow } from './AgentRunRow';

const AgentRunsSection = (props: AgentRunsProps) => {
  const { title, runs, boardKey, now, onLog } = props;
  if (runs.length === 0) return null;
  return (
    <Box role="table" aria-label={title} className="agents-runs">
      <Text as="h3" variant="overline" className="agents-runs__title">{`${title} · ${runs.length}`}</Text>
      {runs.map((run) => <AgentRunRow key={run.id} run={run} boardKey={boardKey} now={now} onLog={onLog} />)}
    </Box>
  );
};

export { AgentRunsSection };
