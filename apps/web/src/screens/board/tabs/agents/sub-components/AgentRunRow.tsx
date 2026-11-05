/* @layer renderer-app @kind component */
import { Box, Flex, Icon, IconButton, Text } from '@drizztdourden08/tessera';
import { RecordRow, recordMenuGroups, useRecordPane } from '../../../../../shell';
import { isLiveRun, runLine } from '../../../../../data/runs';
import { formatAgo } from '../../../../../format';
import { usePlay } from '../../../run-play';
import type { AgentRunRowProps } from '../row-props.type';
import { LOG_PANE_TAB, runSpan } from '../model';
import { PAGE_ORIGIN } from './AgentRunRow.constants';

const detailOf = (props: AgentRunRowProps): string => {
  const { run, now } = props;
  const line = runLine(run);
  if (isLiveRun(run)) return [run.origin ?? '', run.agent ?? '', line.last].filter(Boolean).join(' \u00b7 ');
  const ended = run.endedAt ? `${formatAgo(run.endedAt, now)} ago` : '';
  return [line.label, runSpan(run), ended, run.orphan ? 'orphan' : '', line.last].filter(Boolean).join(' \u00b7 ');
};

const AgentRunRow = (props: AgentRunRowProps) => {
  const { run, boardKey, now } = props;
  const pane = useRecordPane();
  const openLog = () => pane.open(run.id, LOG_PANE_TAB);
  const play = usePlay(boardKey);
  const live = isLiveRun(run);
  const stop = () => void play.stop(run);
  const line = runLine(run);
  return (
    <Box data-swap-item>
      <RecordRow
        head={{
          kind: 'run',
          code: run.id,
          assignee: run.origin === PAGE_ORIGIN ? 'owner' : '',
          boardKey,
          threads: [],
          menu: recordMenuGroups('run', run.id, { open: openLog, stop: live ? stop : undefined }),
          menuLabel: `More actions for ${run.id}`,
        }}
        title={run.name}
        chips={[{ id: 'status', text: line.label, title: detailOf(props) }, ...(live ? [{ id: 'span', text: runSpan(run, now) }] : [])]}
        trailing={
          <Flex gap="xs" align="center">
            <Text as="span" variant="caption" tone="muted" clamp={1}>{line.last}</Text>
            {live && (
              <IconButton label={`Stop run ${run.id}`} variant="ghost" tone="danger" size="2xs" disabled={play.busy} onClick={stop}>
                <Icon name="square" />
              </IconButton>
            )}
          </Flex>
        }
        onOpen={openLog}
      />
    </Box>
  );
};

export { AgentRunRow };
