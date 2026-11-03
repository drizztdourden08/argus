/* @layer renderer-app @kind component */
import { Flex, Spacer, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import type { FeedFactsProps } from '../OwnerFeed.type';
import { ItemChip } from './ItemChip';

const FeedFacts = (props: FeedFactsProps) => {
  const { facts, index } = props;
  return (
    <Flex gap="sm" align="center">
      {facts.item ? <ItemChip id={facts.item} index={index} /> : null}
      <Spacer />
      <Text variant="caption" tone="muted">
        {facts.thread ? `${facts.thread} · ` : null}
        <Age value={facts.at} />
      </Text>
    </Flex>
  );
};

export { FeedFacts };
