/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { LauncherPartProps } from '../builds-props.type';
import { UsageLine } from './UsageLine';

const LauncherMeta = (props: LauncherPartProps) => {
  const { view } = props;
  const { launcher, uses } = view;
  return (
    <Flex gap="md" align="center" wrap>
      {launcher.branch && <Text variant="caption" tone="muted" title="Branch">{'branch '}<Text.Code>{launcher.branch}</Text.Code></Text>}
      {launcher.commit && <Text variant="caption" tone="muted" title="Commit">{'commit '}<Text.Code>{launcher.commit}</Text.Code></Text>}
      {launcher.cwd && <Text variant="caption" tone="muted" title="Working folder">{'run in '}<Text.Code>{launcher.cwd}</Text.Code></Text>}
      {uses.length > 0 && <Text variant="caption" tone="muted">{`started by bundle ${uses.join(', ')}`}</Text>}
      <UsageLine view={view} />
    </Flex>
  );
};

export { LauncherMeta };
