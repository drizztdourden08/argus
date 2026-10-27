/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { isLiveRun, runsOf } from '../../../data/runs';
import { usePlay } from './behavior/usePlay';
import type { RunPlayProps } from './run-play.type';
import { PlayDisabled } from './sub-components/PlayDisabled';
import { RunStatus } from './sub-components/RunStatus';

const RunPlay = (props: RunPlayProps) => {
  const { label, def, hint, boardKey, runs } = props;
  const play = usePlay(boardKey);
  if (!def) return <PlayDisabled label={label} hint={hint} />;
  const mine = runsOf(runs, def.name);
  const live = mine.find(isLiveRun);
  const shown = live ?? mine[0];
  return (
    <Flex gap="sm" align="center" wrap>
      {live ? (
        <Button size="sm" variant="danger" disabled={play.busy} aria-label={`Stop ${live.id}`} onClick={() => void play.stop(live)}>Stop</Button>
      ) : (
        <Button size="sm" disabled={play.busy} aria-label={`Play ${label}`} onClick={() => void play.start(def, label)}>Play</Button>
      )}
      {shown && <RunStatus run={shown} />}
    </Flex>
  );
};

export { RunPlay };
