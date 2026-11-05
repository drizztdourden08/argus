/* @layer renderer-app @kind component */
import { isLiveRun, runsOf } from '../../../../../data/runs';
import { usePlay } from '../../../run-play';
import { useTestsShared } from '../behavior/useTestsShared';
import { playHint } from '../model';
import type { RunActionsProps } from '../tests-head-props.type';
import { TipButton } from './TipButton';

const RunPlayButton = (props: RunActionsProps) => {
  const { view, runs } = props;
  const { boardKey } = useTestsShared();
  const play = usePlay(boardKey);
  const { launch, item, build } = view;
  if (!build || view.stage === 'draft') return null;
  const def = launch.def;
  if (!def) return <TipButton icon="play" label={`Play ${build}`} tip={playHint(launch, item.id)} disabled />;
  const live = runsOf(runs, def.name).find(isLiveRun);
  if (live) {
    return <TipButton icon="square" tone="danger" label={`Stop ${live.id}`} tip={`Stop ${live.id}`} disabled={play.busy} onClick={() => void play.stop(live)} />;
  }
  return <TipButton icon="play" tone="accent" label={`Play ${build}`} tip={`Run ${launch.name ?? build}`} disabled={play.busy} onClick={() => void play.start(def, build)} />;
};

export { RunPlayButton };
