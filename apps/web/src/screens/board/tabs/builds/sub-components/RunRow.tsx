/* @layer renderer-app @kind component */
import { Button, DateText, Flex, ListItemRow, Status, Tag, Text } from '@drizztdourden08/tessera';
import type { ListItemColumn } from '@drizztdourden08/tessera';
import { isLiveRun, runLine } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';
import type { RunRowProps } from '../builds-props.type';
import { runDuration, runNote } from '../model';
import { RunStopButton } from './RunStopButton';

const columnsOf = (run: RunEntry): ListItemColumn[] => {
  const line = runLine(run);
  const live = isLiveRun(run);
  return [
    { primary: <Status tone={line.tone} dot pulse={run.status === 'running'}>{line.label}</Status>, secondary: run.orphan ? 'orphan' : undefined },
    {
      primary: live ? <DateText value={run.startedAt ?? run.queuedAt} format="elapsed" /> : runDuration(run) || '-',
      secondary: live || !run.endedAt ? undefined : <DateText value={run.endedAt} format="relative" />,
      align: 'end',
    },
  ];
};

const RunRow = (props: RunRowProps) => {
  const { run, boardKey, onLog } = props;
  return (
    <ListItemRow
      name={<Flex gap="xs" align="center"><Text mono weight={600}>{run.name}</Text><Tag>{run.id}</Tag></Flex>}
      meta={<Text variant="caption" tone="dim" mono>{runNote(run)}</Text>}
      columns={columnsOf(run)}
      action={(
        <Flex gap="xs">
          {isLiveRun(run) && <RunStopButton run={run} boardKey={boardKey} />}
          <Button size="sm" variant="secondary" aria-label={`Open log of ${run.id}`} onClick={() => onLog(run.id)}>Log</Button>
        </Flex>
      )}
      data={{ 'data-run-id': run.id }}
    />
  );
};

export { RunRow };
