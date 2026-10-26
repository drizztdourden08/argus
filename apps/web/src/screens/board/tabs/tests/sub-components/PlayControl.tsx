/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { usePlay } from '../behavior/usePlay';
import { isLiveRun, playHint, runsOf } from '../model';
import type { PlayControlProps } from '../tests-props.type';
import { PlayDisabled } from './PlayDisabled';
import { RunStatus } from './RunStatus';

const PlayControl = (props: PlayControlProps) => {
  const { tag, launch, boardKey, runs } = props;
  const play = usePlay(boardKey);
  const { def } = launch;
  if (!def) return <PlayDisabled tag={tag} hint={playHint(launch, tag)} />;
  const mine = runsOf(runs, def.name);
  const live = mine.find(isLiveRun);
  const shown = live ?? mine[0];
  return (
    <Flex gap="sm" align="center" wrap>
      {live ? (
        <Button size="sm" variant="danger" disabled={play.busy} aria-label={`Stop ${live.id}`} onClick={() => void play.stop(live)}>Stop</Button>
      ) : (
        <Button size="sm" variant="primary" disabled={play.busy} aria-label={`Play ${tag}`} onClick={() => void play.start(def, tag)}>Play</Button>
      )}
      {shown && <RunStatus run={shown} />}
    </Flex>
  );
};

export { PlayControl };
