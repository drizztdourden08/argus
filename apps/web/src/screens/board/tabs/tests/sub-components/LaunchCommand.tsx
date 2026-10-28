/* @layer renderer-app @kind component */
import { Box, Button, Callout, Flex, Icon, Link, Tag, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { RunPlay } from '../../../run-play';
import { LogButton, RunTerminal } from '../../../run-terminal';
import { copyText } from '../../../shared';
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
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center" wrap>
        <Box surface="sunken" className="bundle__command" title={launch.cwd ? `Run in ${launch.cwd}` : 'Launch command'}>
          <Text mono>{launch.cmd}</Text>
        </Box>
        <Button size="sm" variant="tertiary" title="Copy the command line" onClick={() => copyText(launch.cmd)}>Copy</Button>
        {!draft && <RunPlay label={tag} def={launch.def} hint={playHint(launch, tag)} boardKey={boardKey} runs={runs} />}
        <LogButton run={shown} onLog={onLog} />
      </Flex>
      {launch.cwd && <Text variant="caption" tone="muted">run in <Text.Code>{launch.cwd}</Text.Code></Text>}
      {launch.name && (
        <Flex gap="xs" align="center">
          <Text variant="caption" tone="muted">from</Text>
          <Tag><Link href="#builds" tone="neutral" variant="subtle"><Icon name="link" size={12} /> launcher {launch.name}</Link></Tag>
        </Flex>
      )}
      {shown && <RunTerminal key={shown.id} run={shown} boardKey={boardKey} onLog={onLog} />}
    </Flex>
  );
};

export { LaunchCommand };
