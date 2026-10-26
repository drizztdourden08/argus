/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { PurgeCountsViewProps } from '../trash-props.type';
import { PURGE_LABEL } from '../trash';

const PurgeCountsView = (props: PurgeCountsViewProps) => {
  const { counts } = props;
  return (
    <Flex gap="md" wrap role="group" aria-label="What a purge removes">
      {Object.entries(PURGE_LABEL).map(([key, label]) => (
        <Flex key={key} direction="column" align="center">
          <Text weight={600}>{String(counts[key as keyof typeof counts])}</Text>
          <Text variant="caption" tone="muted">{label}</Text>
        </Flex>
      ))}
    </Flex>
  );
};

export { PurgeCountsView };
