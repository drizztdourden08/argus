/* @layer renderer-app @kind component */
import { Box, Button, Flex, Icon, IconButton, Tag, Text } from '@drizztdourden08/tessera';
import { isLiveRun, runLine } from '../../../../../data/runs';
import { formatAgo } from '../../../../../format';
import { usePlay } from '../../../run-play';
import type { AgentRunRowProps } from '../row-props.type';
import { runSpan, tagLook } from '../model';

const detailOf = (props: AgentRunRowProps): string => {
  const { run, now } = props;
  const line = runLine(run);
  if (isLiveRun(run)) return [run.origin ?? '', run.agent ?? '', line.last].filter(Boolean).join(' · ');
  const ended = run.endedAt ? `${formatAgo(run.endedAt, now)} ago` : '';
  return [line.label, runSpan(run), ended, run.orphan ? 'orphan' : '', line.last].filter(Boolean).join(' · ');
};

const AgentRunRow = (props: AgentRunRowProps) => {
  const { run, boardKey, now, onLog } = props;
  const play = usePlay(boardKey);
  const live = isLiveRun(run);
  return (
    <Box role="row" className="agents-run" data-run-id={run.id}>
      <Text as="span" mono weight={600} className="agents-run__name" title={run.id}>
        {run.name}
        {' '}
        <Text as="span" tone="muted">{run.id}</Text>
      </Text>
      <Flex inline gap="sm" align="center">
        <Tag {...tagLook(runLine(run).tone)}>{run.status}</Tag>
        {live && <Text as="span">{runSpan(run, now)}</Text>}
      </Flex>
      <Text as="span" tone="muted" className="agents-run__detail" title={detailOf(props)}>{detailOf(props)}</Text>
      <Flex gap="2xs" align="center" justify="end">
        {live && (
          <IconButton label={`Stop run ${run.id}`} variant="ghost" tone="danger" size="sm" disabled={play.busy} onClick={() => void play.stop(run)}>
            <Icon name="square" />
          </IconButton>
        )}
        <Button size="sm" variant="ghost" aria-label={`Log of ${run.id}`} onClick={() => onLog(run.id)}>Log</Button>
      </Flex>
    </Box>
  );
};

export { AgentRunRow };
