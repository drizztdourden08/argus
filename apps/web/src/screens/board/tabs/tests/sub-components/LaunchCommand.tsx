/* @layer renderer-app @kind component */
import { Callout, CopyValue, Flex, Link, Text } from '@drizztdourden08/tessera';
import type { LaunchCommandProps } from '../tests-props.type';
import { PlayControl } from './PlayControl';
import { PlayDisabled } from './PlayDisabled';

const LaunchCommand = (props: LaunchCommandProps) => {
  const { view, boardKey, runs, draft } = props;
  const { launch, group } = view;
  const { tag } = group;
  if (!launch.cmd && launch.gone) {
    return (
      <Flex gap="sm" align="center" wrap>
        <Callout tone="warning">{launch.gone}. The thread links another with coord build set {tag} --launcher NAME.</Callout>
        <PlayDisabled tag={tag} hint={launch.gone} />
      </Flex>
    );
  }
  if (!launch.cmd) {
    return <Text tone="muted">no launch command yet: the thread links a launcher with coord build set {tag} --launcher NAME (or sets one with --cmd).</Text>;
  }
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="sm" align="center" wrap>
        <CopyValue value={launch.cmd} label="Launch command" mono />
        {!draft && <PlayControl tag={tag} launch={launch} boardKey={boardKey} runs={runs} />}
      </Flex>
      {launch.cwd && <Text variant="caption" tone="muted">run in <Text.Code>{launch.cwd}</Text.Code></Text>}
      {launch.name && <Text variant="caption" tone="muted">from <Link href="#builds">launcher {launch.name}</Link></Text>}
    </Flex>
  );
};

export { LaunchCommand };
