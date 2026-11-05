/* @layer renderer-app @kind component */
import './RunCard.css';
import './SectionHead.css';
import { Card, Flex, Text } from '@drizztdourden08/tessera';
import type { ClosedRunCardProps } from '../tests-props.type';
import { ClosedTestRow } from './ClosedTestRow';
import { RunCounts } from './RunCounts';
import { RunFrame } from './RunFrame';

const ClosedRunCard = (props: ClosedRunCardProps) => {
  const { view, open, onToggle } = props;
  return (
    <Card flush className="run-card run-card--closed" data-swap-item data-run={view.item.id}>
      <RunFrame view={view} muted open={open} onToggle={onToggle} content={<Flex gap="xs" align="center" className="run-card__end"><RunCounts counts={view.counts} /></Flex>}>
        {view.tests.length > 0
          ? view.tests.map((test) => <ClosedTestRow key={test.id} test={test} />)
          : <Text tone="muted" className="test-section__empty">No test in this run.</Text>}
      </RunFrame>
    </Card>
  );
};

export { ClosedRunCard };
