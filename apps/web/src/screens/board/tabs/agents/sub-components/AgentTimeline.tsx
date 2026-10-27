/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { AgentTimelineProps } from '../drawer-props.type';
import { agentElapsed, isActive } from '../model';

const when = (iso: string | null | undefined, now: number) =>
  iso ? (
    <>
      {new Date(iso).toLocaleString()}
      {' '}
      <Text as="span" variant="caption">{`(${formatAgo(iso, now)} ago)`}</Text>
    </>
  ) : '-';

const AgentTimeline = (props: AgentTimelineProps) => {
  const { agent, now } = props;
  return (
    <Box as="dl" className="agents-timeline">
      <Box as="dt" className="agents-timeline__label">Started</Box>
      <Box as="dd">{when(agent.startedAt, now)}</Box>
      <Box as="dt" className="agents-timeline__label">Updated</Box>
      <Box as="dd">{when(agent.updatedAt, now)}</Box>
      <Box as="dt" className="agents-timeline__label">Finished</Box>
      <Box as="dd">{when(agent.finishedAt, now)}</Box>
      <Box as="dt" className="agents-timeline__label">{isActive(agent) ? 'Elapsed' : 'Duration'}</Box>
      <Box as="dd">{agentElapsed(agent, now)}</Box>
    </Box>
  );
};

export { AgentTimeline };
