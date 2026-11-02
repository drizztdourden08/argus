/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { DependencyFactsProps } from '../drawer-props.type';

const DependencyFacts = (props: DependencyFactsProps) => {
  const { view, now } = props;
  return (
    <Flex direction="column" gap="xs">
      {view.thread && <Text variant="caption" tone="muted">{`Thread ${view.thread}`}</Text>}
      {view.since && <Text variant="caption" tone="muted">{`Open for ${formatAgo(view.since, now)}`}</Text>}
      {view.reason && <Text variant="caption">{`Reason: ${view.reason}`}</Text>}
    </Flex>
  );
};

export { DependencyFacts };
