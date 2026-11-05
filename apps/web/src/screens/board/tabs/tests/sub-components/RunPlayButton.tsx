/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { isLiveRun, runsOf } from '../../../../../data/runs';
import { usePlay } from '../../../run-play';
import { useTestsShared } from '../behavior/useTestsShared';
import { playHint } from '../model';
import type { RunActionsProps } from '../tests-head-props.type';

const RunPlayButton = (props: RunActionsProps) => {
  const { view, runs } = props;
  const { boardKey } = useTestsShared();
  const play = usePlay(boardKey);
  const { launch, item, build } = view;
  if (!build || view.stage === 'draft') return null;
  const def = launch.def;
  if (!def) return <IconButton size="xs" label={`Play ${build}`} tooltip={playHint(launch, item.id)} disabled><Icon name="play" /></IconButton>;
  const live = runsOf(runs, def.name).find(isLiveRun);
  if (live) {
    return (
      <IconButton size="xs" tone="danger" label={`Stop ${live.id}`} tooltip={`Stop ${live.id}`} disabled={play.busy} onClick={() => void play.stop(live)}>
        <Icon name="square" />
      </IconButton>
    );
  }
  return (
    <IconButton size="xs" tone="accent" label={`Play ${build}`} tooltip={`Run ${launch.name ?? build}`} disabled={play.busy} onClick={() => void play.start(def, build)}>
      <Icon name="play" />
    </IconButton>
  );
};

export { RunPlayButton };
