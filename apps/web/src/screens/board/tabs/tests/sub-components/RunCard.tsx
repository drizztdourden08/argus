/* @layer renderer-app @kind component */
import './RunCard.css';
import { Card, Disclosure } from '@drizztdourden08/tessera';
import type { RunCardProps } from '../tests-props.type';
import { RunActions } from './RunActions';
import { RunTitle } from './RunTitle';
import { TestSections } from './TestSections';

const RunCard = (props: RunCardProps) => {
  const { view, runs, open, onToggle } = props;
  return (
    <Card flush className="run-card" data-swap-item edgeTone={view.handed ? 'neutral' : undefined} data-run={view.item.id}>
      <Disclosure open={open} onOpenChange={onToggle} summary={<RunTitle view={view} />} end={<RunActions view={view} runs={runs} />} wrapEnd>
        <TestSections tests={view.tests} emptyText="No test in this run yet." />
      </Disclosure>
    </Card>
  );
};

export { RunCard };
