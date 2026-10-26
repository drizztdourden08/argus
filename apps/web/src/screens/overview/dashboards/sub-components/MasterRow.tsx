/* @layer renderer-app @kind component */
import { Button, Inline, Text, Tooltip } from '@drizztdourden08/tessera';
import { ShortAge } from '../../sub-components/ShortAge';
import { useMasterRun } from '../behavior/useMasterRun';
import { NO_PLAY } from '../dashboards.constants';
import type { MasterRowProps } from '../dashboards.type';
import { masterLine } from '../model';

const MasterRow = (props: MasterRowProps) => {
  const { card } = props;
  const run = useMasterRun(card);
  const line = masterLine(card);
  const master = card.master;
  if (!master) return <Text variant="caption" tone={line.tone}>{line.head}</Text>;
  return (
    <Inline gap="sm" align="center" wrap>
      <Tooltip content={[master.title, master.cmd].filter(Boolean).join('\n') || 'master'}>
        <Text variant="label">master</Text>
      </Tooltip>
      {line.running
        ? <Button size="xs" variant="danger" disabled={run.busy} onClick={run.stop}>Stop</Button>
        : (
          <Button size="xs" disabled={run.busy || !master.play} onClick={run.play} title={master.play ? `Run ${card.name} master from the dashboard (the output is on its board)` : NO_PLAY}>
            Play
          </Button>
        )}
      {line.head && (
        <Text variant="caption" tone={line.tone}>
          {line.head}
          {line.at ? <> <ShortAge value={line.at} ago={!line.running} /></> : null}
          {line.tail}
        </Text>
      )}
    </Inline>
  );
};

export { MasterRow };
