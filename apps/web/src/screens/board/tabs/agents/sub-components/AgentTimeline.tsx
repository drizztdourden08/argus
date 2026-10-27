/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { AgentTimelineProps } from '../AgentsTab.type';
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
      <Text as="dt" variant="caption">Started</Text>
      <Text as="dd">{when(agent.startedAt, now)}</Text>
      <Text as="dt" variant="caption">Updated</Text>
      <Text as="dd">{when(agent.updatedAt, now)}</Text>
      <Text as="dt" variant="caption">Finished</Text>
      <Text as="dd">{when(agent.finishedAt, now)}</Text>
      <Text as="dt" variant="caption">{isActive(agent) ? 'Elapsed' : 'Duration'}</Text>
      <Text as="dd">{agentElapsed(agent, now)}</Text>
    </Box>
  );
};

export { AgentTimeline };
