/* @layer renderer-app @kind component */
import './RunCard.css';
import { Card, Flex, Text } from '@drizztdourden08/tessera';
import { RecordFrame } from '../../../../../shell';
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
      <RecordFrame
        name={card.key}
        kind={item ? (item.type ?? 'task') : undefined}
        density="row"
        defaultOpen
        code={<Text font="title" className="run-card__tag">{card.key}</Text>}
        title={item?.title && <Text tone="dim" smartTimes className="run-card__title">{item.title}</Text>}
        content={
          <Flex gap="xs" align="center" className="run-card__end">
            <RunCounts counts={card.counts} />
            <RunProgress counts={card.counts} />
          </Flex>
        }
      >
        <TestSections tests={card.tests} emptyText="none" />
      </RecordFrame>
    </Card>
  );
};

export { TestCard };
