/* @layer renderer-app @kind component */
import { DateText, Text } from '@drizztdourden08/tessera';
import type { LauncherPartProps } from '../builds-props.type';

const UsageLine = (props: LauncherPartProps) => {
  const { view } = props;
  const { usage, launcher } = view;
  if (!usage) return null;
  const every = `expected every ${usage.every}${usage.legacy ? ', default' : ''}`;
  const title = usage.legacy
    ? `The thread has not said how often it is used (coord launcher set ${launcher.name} --use-every 7d): 7d is assumed.`
    : `How often it is expected to be used (coord launcher set ${launcher.name} --use-every)`;
  return (
    <Text variant="caption" tone={usage.stale ? 'warning' : 'muted'} title={title}>
      {usage.stale ? 'unused ' : 'last used '}
      <DateText value={usage.lastUsed} format="relative" />
      {` (${every})`}
    </Text>
  );
};

export { UsageLine };
