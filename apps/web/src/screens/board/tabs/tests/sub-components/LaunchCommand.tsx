/* @layer renderer-app @kind component */
import { Button, ButtonRow, Callout, Flex, Icon, Link, Tag, Text, TextInput } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { RunPlay } from '../../../run-play';
import { LogButton, RunTerminal } from '../../../run-terminal';
import { copyText } from '../../../shared';
import { playHint, shownRun } from '../model';
import type { LaunchCommandProps } from '../tests-props.type';

const GoneLaunch = (props: LaunchCommandProps) => {
  const { view, runs } = props;
  const { boardKey } = useTestsShared();
  const { launch, item } = view;
  return (
    <Flex gap="sm" align="center" wrap>
      <Callout tone="warning">{launch.gone}. The thread sets another build on {item.id}.</Callout>
      <RunPlay label={view.build ?? item.id} def={null} hint={launch.gone} boardKey={boardKey} runs={runs} />
    </Flex>
  );
};

const LaunchFrom = (props: { name: string }) => (
  <Flex gap="xs" align="center">
    <Text variant="caption" tone="muted">from</Text>
    <Tag><Link href="#builds" tone="neutral" variant="subtle"><Icon name="link" size={12} /> launcher {props.name}</Link></Tag>
  </Flex>
);

const LaunchCommand = (props: LaunchCommandProps) => {
  const { view, runs, draft } = props;
  const { boardKey, onLog } = useTestsShared();
  const { launch, item } = view;
  if (!launch.cmd && launch.gone) return <GoneLaunch {...props} />;
  if (!launch.cmd) {
    return <Text tone="muted">no launch command yet: the thread sets a build with a launcher on {item.id}.</Text>;
  }
  const shown = draft ? null : shownRun(launch, runs);
  return (
    <Flex direction="column" gap="xs">
      <ButtonRow
        align="start"
        lead={<TextInput readOnly mono grow size="sm" aria-label="Launch command" title={launch.cwd ? `Run in ${launch.cwd}` : 'Launch command'} value={launch.cmd} />}
      >
        <Button size="sm" variant="quiet" title="Copy the command line" onClick={() => copyText(launch.cmd)}>Copy</Button>
        {!draft && <RunPlay label={view.build ?? item.id} def={launch.def} hint={playHint(launch, item.id)} boardKey={boardKey} runs={runs} />}
        <LogButton run={shown} onLog={onLog} />
      </ButtonRow>
      {launch.cwd && <Text variant="caption" tone="muted">run in <Text.Code>{launch.cwd}</Text.Code></Text>}
      {launch.name && <LaunchFrom name={launch.name} />}
      {shown && <RunTerminal key={shown.id} run={shown} boardKey={boardKey} onLog={onLog} />}
    </Flex>
  );
};

export { LaunchCommand };
