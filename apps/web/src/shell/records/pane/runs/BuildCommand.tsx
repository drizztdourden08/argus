/* @layer renderer-app @kind component */
import { Button, ButtonRow, Flex, Text, TextInput } from '@drizztdourden08/tessera';
import { noPlayHint, SHELL_NAME } from '../../../../screens/board/tabs/builds';
import { RunPlay } from '../../../../screens/board/run-play';
import { copyText } from '../../../../screens/board/shared';
import type { BuildCommandProps } from './runs.type';

const BuildCommand = (props: BuildCommandProps) => {
  const { view, boardKey, runs } = props;
  const { launcher } = view;
  const cmd = launcher.cmd ?? '';
  const tip = launcher.cwd ? `${cmd}\nin ${launcher.cwd}` : cmd;
  return (
    <Flex direction="column" gap="2xs">
      <Text variant="caption" tone="muted">
        {`Command line · run by ${SHELL_NAME} from `}
        {launcher.cwd ? <Text variant="caption" tone="dim" mono>{launcher.cwd}</Text> : 'the checkout'}
      </Text>
      <ButtonRow align="start" lead={<TextInput readOnly mono grow size="sm" aria-label="Command line" title={tip} value={cmd} />}>
        <Button size="sm" variant="quiet" title="Copy the command line" onClick={() => copyText(cmd)}>Copy</Button>
        {view.playable && <RunPlay label={launcher.name} def={view.def} hint={noPlayHint(launcher.name)} boardKey={boardKey} runs={runs} />}
      </ButtonRow>
    </Flex>
  );
};

export { BuildCommand };
