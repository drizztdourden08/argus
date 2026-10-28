/* @layer renderer-app @kind component */
import { Button, ButtonRow, Flex, Text, TextInput } from '@drizztdourden08/tessera';
import { RunPlay } from '../../../run-play';
import { LogButton } from '../../../run-terminal';
import { copyText } from '../../../shared';
import { SHELL_NAME } from '../builds.constants';
import type { LauncherCardProps } from '../builds-props.type';
import { noPlayHint } from '../model';

const LauncherCommand = (props: LauncherCardProps) => {
  const { view, boardKey, runs, onLog } = props;
  const { launcher } = view;
  const cmd = launcher.cmd ?? '';
  const tip = launcher.cwd ? `${cmd}\nin ${launcher.cwd}` : cmd;
  return (
    <Flex direction="column" gap="2xs">
      <Text variant="caption" tone="muted">
        {`Command line · run by ${SHELL_NAME} from `}
        {launcher.cwd ? <Text variant="caption" tone="dim" mono>{launcher.cwd}</Text> : 'the checkout'}
      </Text>
      <ButtonRow
        align="start"
        lead={<TextInput readOnly mono grow size="sm" aria-label="Command line" title={tip} value={cmd} />}
      >
        <Button size="sm" variant="tertiary" title="Copy the command line" onClick={() => copyText(cmd)}>Copy</Button>
        {view.playable && <RunPlay label={launcher.name} def={view.def} hint={noPlayHint(launcher.name)} boardKey={boardKey} runs={runs} />}
        {view.playable && <LogButton run={view.shown} onLog={onLog} />}
      </ButtonRow>
    </Flex>
  );
};

export { LauncherCommand };
