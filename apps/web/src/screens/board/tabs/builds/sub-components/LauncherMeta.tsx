/* @layer renderer-app @kind component */
import { Flex, Icon, Text } from '@drizztdourden08/tessera';
import type { LauncherPartProps } from '../builds-props.type';
import { UsageLine } from './UsageLine';

const LauncherMeta = (props: LauncherPartProps) => {
  const { view } = props;
  const { launcher, uses, usage } = view;
  if (!launcher.branch && !launcher.commit && !launcher.cwd && uses.length === 0 && !usage) return null;
  return (
    <Flex gap="md" align="center" wrap>
      {launcher.branch && (
        <Text variant="caption" tone="muted" title="Branch">
          <Icon name="git-fork" size={12} aria-hidden />
          {` ${launcher.branch}`}
        </Text>
      )}
      {launcher.commit && <Text variant="caption" tone="muted" title="Commit">{'commit '}<Text variant="caption" tone="dim" mono>{launcher.commit}</Text></Text>}
      {launcher.cwd && <Text variant="caption" tone="muted" title="Working folder">{'run in '}<Text variant="caption" tone="dim" mono>{launcher.cwd}</Text></Text>}
      {uses.length > 0 && <Text variant="caption" tone="muted">{`started by bundle ${uses.join(', ')}`}</Text>}
      <UsageLine view={view} />
    </Flex>
  );
};

export { LauncherMeta };
