/* @layer renderer-app @kind component */
import { CopyValue, Flex, Text } from '@drizztdourden08/tessera';
import { RunPlay } from '../../../run-play';
import { LogButton } from '../../../run-terminal';
import type { LauncherCardProps } from '../builds-props.type';
import { noPlayHint } from '../model';

const LauncherCommand = (props: LauncherCardProps) => {
  const { view, boardKey, runs, onLog } = props;
  const { launcher } = view;
  return (
    <Flex direction="column" gap="2xs">
      <Text variant="caption" tone="muted">
        {'Command line, run from '}
        {launcher.cwd ? <Text.Code>{launcher.cwd}</Text.Code> : 'the checkout'}
      </Text>
      <Flex gap="sm" align="center" wrap>
        <CopyValue value={launcher.cmd ?? ''} label="Command line" mono />
        {view.playable && <RunPlay label={launcher.name} def={view.def} hint={noPlayHint(launcher.name)} boardKey={boardKey} runs={runs} />}
        {view.playable && <LogButton run={view.shown} onLog={onLog} />}
      </Flex>
    </Flex>
  );
};

export { LauncherCommand };
