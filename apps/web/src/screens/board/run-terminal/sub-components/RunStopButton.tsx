/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { usePlay } from '../../run-play';
import type { RunStopButtonProps } from '../run-terminal.type';

const RunStopButton = (props: RunStopButtonProps) => {
  const { run, boardKey } = props;
  const play = usePlay(boardKey);
  return <Button size="sm" variant="danger" disabled={play.busy} aria-label={`Stop run ${run.id}`} onClick={() => void play.stop(run)}>Stop</Button>;
};

export { RunStopButton };
