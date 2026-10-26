/* @layer renderer-app @kind component */
import { Card, Flex, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { countsText, NO_KEY, threadName } from '../model';
import type { TestCardProps } from '../tests-props.type';
import { BundleProgress } from './BundleProgress';
import { TestSections } from './TestSections';

const TestCard = (props: TestCardProps) => {
  const { card } = props;
  const { group, threads } = useTestsShared();
  const label = group === 'thread' && card.key !== NO_KEY ? threadName(threads, card.key) : card.key;
  return (
    <Card data-group={card.key}>
      <Flex direction="column" gap="md">
        <Flex gap="sm" align="center" wrap>
          <Text mono weight={600}>{label}</Text>
          {card.item?.title && <Text>{card.item.title}</Text>}
          <Text variant="caption" tone="muted">{countsText(card.counts)}</Text>
          <BundleProgress counts={card.counts} />
        </Flex>
        <TestSections tests={card.tests} emptyText="none" />
      </Flex>
    </Card>
  );
};

export { TestCard };
