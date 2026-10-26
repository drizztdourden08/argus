/* @layer renderer-app @kind component */
import { Button, LogPanel } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { isLiveRun, useRunLog } from '../../../../../data/runs';
import { useStdin } from '../behavior/useStdin';
import { TERMINAL_KINDS } from '../builds.constants';
import type { TerminalBodyProps } from '../builds-props.type';
import { logRows } from '../model';

const TerminalBody = (props: TerminalBodyProps) => {
  const { run, boardKey, limit, height } = props;
  const log = useRunLog(boardKey, run.id, { limit, live: isLiveRun(run), enabled: true });
  const rows = useMemo(() => logRows(log.lines), [log.lines]);
  const send = useStdin(boardKey, run.id);
  const typing = run.status === 'running' && run.stdin === true;
  return (
    <LogPanel
      rows={rows}
      kinds={TERMINAL_KINDS}
      ansi
      follow
      height={height}
      prompt=">"
      emptyLabel={isLiveRun(run) ? 'Waiting for the first output line.' : 'No output.'}
      onInput={typing ? send : undefined}
      toolbarExtra={<Button size="sm" variant="ghost" title="Clear what is drawn here (the log keeps everything)" onClick={log.clear}>Clear view</Button>}
    />
  );
};

export { TerminalBody };
