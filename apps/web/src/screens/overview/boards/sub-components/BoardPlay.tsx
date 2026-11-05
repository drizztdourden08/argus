/* @layer renderer-app @kind component */
import { Button, Inline, Text, Tooltip } from '@drizztdourden08/tessera';
import { NO_PLAY } from '../boards-run.constants';
import { masterLine } from '../model';
import { MasterStatus } from './MasterStatus';
import type { BoardPlayProps } from '../boards.type';

const BoardPlay = (props: BoardPlayProps) => {
  const { card, run, onLog } = props;
  const line = masterLine(card);
  const { master } = card;
  const last = master?.lastRun;
  if (!master) return <Text variant="caption" tone={line.tone === 'neutral' ? 'muted' : line.tone}>{line.head}</Text>;
  return (
    <Inline gap="xs" align="center" wrap>
      <Tooltip content={[master.title, master.cmd].filter(Boolean).join('\n') || 'master'}>
        <Text variant="caption" tone="dim" mono weight={600}>master</Text>
      </Tooltip>
      {line.running
        ? <Button size="xs" variant="danger" disabled={run.busy} onClick={run.stop}>Stop</Button>
        : (
          <Button size="xs" variant="primary" fill="solid" disabled={run.busy || !master.play} onClick={run.play} title={master.play ? `Run ${card.name} master from the dashboard (the output is on its board)` : NO_PLAY}>
            Play
          </Button>
        )}
      {last && <MasterStatus line={line} run={last} onLog={(logged) => onLog(card.key, logged)} />}
    </Inline>
  );
};

export { BoardPlay };
