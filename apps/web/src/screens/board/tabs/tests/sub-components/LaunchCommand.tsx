/* @layer renderer-app @kind component */
import { Callout, CopyValue, Flex, Link, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { RunPlay } from '../../../run-play';
import { LogButton } from '../../builds/sub-components/LogButton';
import { RunTerminal } from '../../builds/sub-components/RunTerminal';
import { playHint, shownRun } from '../model';
import type { LaunchCommandProps } from '../tests-props.type';

const LaunchCommand = (props: LaunchCommandProps) => {
  const { view, runs, draft } = props;
  const { boardKey, onLog } = useTestsShared();
  const { launch, group } = view;
  const { tag } = group;
  if (!launch.cmd && launch.gone) {
    return (
      <Flex gap="sm" align="center" wrap>
        <Callout tone="warning">{launch.gone}. The thread links another with coord build set {tag} --launcher NAME.</Callout>
        <RunPlay label={tag} def={null} hint={launch.gone} boardKey={boardKey} runs={runs} />
      </Flex>
    );
  }
  if (!launch.cmd) {
    return <Text tone="muted">no launch command yet: the thread links a launcher with coord build set {tag} --launcher NAME (or sets one with --cmd).</Text>;
  }
  const shown = draft ? null : shownRun(launch, runs);
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="sm" align="center" wrap>
        <CopyValue value={launch.cmd} label="Launch command" mono />
        {!draft && <RunPlay label={tag} def={launch.def} hint={playHint(launch, tag)} boardKey={boardKey} runs={runs} />}
        <LogButton run={shown} onLog={onLog} />
      </Flex>
      {launch.cwd && <Text variant="caption" tone="muted">run in <Text.Code>{launch.cwd}</Text.Code></Text>}
      {launch.name && <Text variant="caption" tone="muted">from <Link href="#builds">launcher {launch.name}</Link></Text>}
      {shown && <RunTerminal key={shown.id} run={shown} boardKey={boardKey} onLog={onLog} />}
    </Flex>
  );
};

export { LaunchCommand };
