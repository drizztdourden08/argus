/* @layer renderer-app @kind component */
import './RunCard.css';
import './SectionHead.css';
import { Text } from '@drizztdourden08/tessera';
import type { ClosedRunCardProps } from '../tests-props.type';
import { ClosedTestRow } from './ClosedTestRow';
import { RunChips } from './RunChips';
import { RunCounts } from './RunCounts';
import { RunFrame } from './RunFrame';

const ClosedRunCard = (props: ClosedRunCardProps) => {
  const { view, open, onToggle } = props;
  return (
    <RunFrame
      view={view}
      open={open}
      onToggle={onToggle}
      muted
      className="run-card run-card--closed"
      signals={(
        <>
          <RunChips view={view} />
          <RunCounts counts={view.counts} />
        </>
      )}
    >
      {view.tests.length > 0
        ? view.tests.map((test) => <ClosedTestRow key={test.id} test={test} />)
        : <Text tone="muted" className="test-section__empty">No test in this run.</Text>}
    </RunFrame>
  );
};

export { ClosedRunCard };
