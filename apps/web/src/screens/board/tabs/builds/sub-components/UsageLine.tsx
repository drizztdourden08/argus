/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import type { LauncherPartProps } from '../builds-props.type';

const UsageLine = (props: LauncherPartProps) => {
  const { view } = props;
  const { usage, launcher } = view;
  if (!usage) return null;
  const title = usage.legacy
    ? `The thread has not said how often it is used (coord launcher set ${launcher.name} --use-every 7d): 7d is assumed.`
    : `How often it is expected to be used (coord launcher set ${launcher.name} --use-every)`;
  const age = <Age value={usage.lastUsed} />;
  return (
    <Text variant="caption" tone={usage.stale ? 'warning' : 'muted'} title={title}>
      {usage.stale
        ? <>{'unused '}{age}{` (expected every ${usage.every}${usage.legacy ? ', default' : ''})`}</>
        : <>{`expected every ${usage.every}${usage.legacy ? ' (default)' : ''} · last used `}{age}{' ago'}</>}
    </Text>
  );
};

export { UsageLine };
