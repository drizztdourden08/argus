/* @layer renderer-app @kind component */
import './RunCard.css';
import { Card } from '@drizztdourden08/tessera';
import type { RunCardProps } from '../tests-props.type';
import { RunActions } from './RunActions';
import { RunFrame } from './RunFrame';
import { TestSections } from './TestSections';

const RunCard = (props: RunCardProps) => {
  const { view, runs, open, onToggle } = props;
  return (
    <Card flush className="run-card" data-swap-item edgeTone={view.handed ? 'neutral' : undefined} data-run={view.item.id}>
      <RunFrame view={view} open={open} onToggle={onToggle} content={<RunActions view={view} runs={runs} />}>
        <TestSections tests={view.tests} emptyText="No test in this run yet." />
      </RunFrame>
    </Card>
  );
};

export { RunCard };
