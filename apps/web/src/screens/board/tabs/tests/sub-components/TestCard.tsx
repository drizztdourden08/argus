/* @layer renderer-app @kind component */
import { Box, Flex, Spacer, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { countsText } from '../model';
import type { TestCardProps } from '../tests-props.type';
import { BundleProgress } from './BundleProgress';
import { TestSections } from './TestSections';

const TestCard = (props: TestCardProps) => {
  const { card } = props;
  const { group } = useTestsShared();
  return (
    <Box className="bundle" data-group={card.key}>
      <Flex gap="sm" align="center" wrap className="bundle__head">
        <Text font="title">{card.key}</Text>
        {group === 'item' && card.item?.title && <Text tone="dim" smartTimes>{card.item.title}</Text>}
        <Spacer />
        <Text variant="caption" tone="dim">{countsText(card.counts)}</Text>
        <BundleProgress counts={card.counts} />
      </Flex>
      <Box className="bundle__body">
        <TestSections tests={card.tests} emptyText="none" />
      </Box>
    </Box>
  );
};

export { TestCard };
