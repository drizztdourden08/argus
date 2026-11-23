/* @layer renderer-app @kind component */
import { Button, Inline, Text, Tooltip } from '@drizztdourden08/tessera';
import { NO_PLAY } from '../boards-run.constants';
import { masterWords } from '../behavior/master-words';
import { masterLine } from '../model';
import { MasterStatus } from './MasterStatus';
import type { BoardPlayProps } from '../boards.type';

const BoardPlay = (props: BoardPlayProps) => {
  const { card, run, now, onLog } = props;
  const { master } = card;
  if (!master) return null;
  const line = masterLine(card);
  const last = master.lastRun;
  return (
    <Inline gap="xs" align="center" wrap>
      <Tooltip content={[master.title, master.cmd].filter(Boolean).join('\n') || 'master'}>
        <Text variant="caption" tone="dim" weight={600}>Master</Text>
      </Tooltip>
      {line.running
        ? <Button size="xs" variant="danger" disabled={run.busy} onClick={run.stop}>Stop</Button>
        : (
          <Button size="xs" variant="primary" fill="solid" disabled={run.busy || !master.play} onClick={run.play} title={master.play ? `Run ${card.name} master from the dashboard (the output is on its board)` : NO_PLAY}>
            Play
          </Button>
        )}
      {last && <MasterStatus line={line} words={masterWords(last, line, now)} run={last} onLog={(logged) => onLog(card.key, logged)} />}
    </Inline>
  );
};

export { BoardPlay };
