/* @layer renderer-app @kind component */
import { Button, Inline, Text, Tooltip } from '@drizztdourden08/tessera';
import { useMasterRun } from '../behavior/useMasterRun';
import { NO_PLAY } from '../dashboards.constants';
import type { MasterRowProps } from '../dashboards.type';
import { masterLine } from '../model';
import { MasterStatus } from './MasterStatus';

const MasterRow = (props: MasterRowProps) => {
  const { card, onLog } = props;
  const run = useMasterRun(card);
  const line = masterLine(card);
  const master = card.master;
  const last = master?.lastRun;
  if (!master) return <Text variant="caption" tone={line.tone === 'neutral' ? 'muted' : line.tone}>{line.head}</Text>;
  return (
    <Inline gap="xs" align="center" wrap>
      <Tooltip content={[master.title, master.cmd].filter(Boolean).join('\n') || 'master'}>
        <Text variant="caption" mono weight={600}>master</Text>
      </Tooltip>
      {line.running
        ? <Button size="xs" variant="danger" disabled={run.busy} onClick={run.stop}>Stop</Button>
        : (
          <Button size="xs" variant="primary" fill="solid" disabled={run.busy || !master.play} onClick={run.play} title={master.play ? `Run ${card.name} master from the dashboard (the output is on its board)` : NO_PLAY}>
            Play
          </Button>
        )}
      {last && <MasterStatus line={line} run={last} onLog={(run) => onLog(card.key, run)} />}
    </Inline>
  );
};

export { MasterRow };
