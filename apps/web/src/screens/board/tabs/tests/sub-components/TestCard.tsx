/* @layer renderer-app @kind component */
import './RunCard.css';
import { Card, Flex, Spacer, Text } from '@drizztdourden08/tessera';
import { TypeIcon } from '../../../../../shell';
import { useTestsShared } from '../behavior/useTestsShared';
import type { TestCardProps } from '../tests-props.type';
import { RunCounts } from './RunCounts';
import { RunProgress } from './RunProgress';
import { TestSections } from './TestSections';

const TestCard = (props: TestCardProps) => {
  const { card } = props;
  const { group } = useTestsShared();
  const item = group === 'item' ? card.item : null;
  return (
    <Card flush className="run-card" data-swap-item data-group={card.key}>
      <Flex gap="xs" align="center" wrap className="run-card__head">
        {item && <TypeIcon kind={item.type ?? 'task'} />}
        <Text font="title" className="run-card__tag">{card.key}</Text>
        {item?.title && <Text tone="dim" smartTimes className="run-card__title">{item.title}</Text>}
        <Spacer />
        <Flex gap="xs" align="center" className="run-card__end">
          <RunCounts counts={card.counts} />
          <RunProgress counts={card.counts} />
        </Flex>
      </Flex>
      <TestSections tests={card.tests} emptyText="none" />
    </Card>
  );
};

export { TestCard };
