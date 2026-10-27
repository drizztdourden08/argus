/* @layer renderer-app @kind component */
import { DateText, Flex, Text } from '@drizztdourden08/tessera';
import type { LauncherPartProps } from '../builds-props.type';
import { LauncherChips } from './LauncherChips';

const LauncherHead = (props: LauncherPartProps) => {
  const { view } = props;
  const { launcher } = view;
  return (
    <Flex gap="sm" align="center" justify="between" wrap>
      <Flex gap="sm" align="center" wrap>
        <Text mono weight={600}>{launcher.name}</Text>
        {launcher.title && <Text tone="dim">{launcher.title}</Text>}
        <LauncherChips view={view} />
      </Flex>
      {launcher.updatedAt && (
        <Text variant="caption" tone="muted">
          {`set by ${launcher.by ?? launcher.createdBy ?? 'a thread'} · `}
          <DateText value={launcher.updatedAt} format="elapsed" length="short" />
          {' ago'}
        </Text>
      )}
    </Flex>
  );
};

export { LauncherHead };
