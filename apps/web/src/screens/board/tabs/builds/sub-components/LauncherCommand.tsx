/* @layer renderer-app @kind component */
import { Box, Button, Flex, Text } from '@drizztdourden08/tessera';
import { RunPlay } from '../../../run-play';
import { LogButton } from '../../../run-terminal';
import { copyText } from '../../../shared';
import { SHELL_NAME } from '../builds.constants';
import type { LauncherCardProps } from '../builds-props.type';
import { noPlayHint } from '../model';
import './LauncherCommand.css';

const LauncherCommand = (props: LauncherCardProps) => {
  const { view, boardKey, runs, onLog } = props;
  const { launcher } = view;
  const cmd = launcher.cmd ?? '';
  return (
    <Flex direction="column" gap="2xs">
      <Text variant="caption" tone="muted">
        {`Command line · run by ${SHELL_NAME} from `}
        {launcher.cwd ? <Text variant="caption" tone="dim" mono>{launcher.cwd}</Text> : 'the checkout'}
      </Text>
      <Flex gap="sm" align="center" wrap>
        <Box surface="sunken" className="launcher-command__code" title={launcher.cwd ? `${cmd}\nin ${launcher.cwd}` : cmd}>{cmd}</Box>
        <Button size="sm" variant="tertiary" title="Copy the command line" onClick={() => copyText(cmd)}>Copy</Button>
        {view.playable && <RunPlay label={launcher.name} def={view.def} hint={noPlayHint(launcher.name)} boardKey={boardKey} runs={runs} />}
        {view.playable && <LogButton run={view.shown} onLog={onLog} />}
      </Flex>
    </Flex>
  );
};

export { LauncherCommand };
