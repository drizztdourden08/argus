/* @layer renderer-app @kind component */
import './RunCard.css';
import { Card, Disclosure } from '@drizztdourden08/tessera';
import type { RunCardProps } from '../tests-props.type';
import { LaunchBox } from './LaunchBox';
import { RunHead } from './RunHead';
import { RunName } from './RunName';
import { TestSections } from './TestSections';

const RunCard = (props: RunCardProps) => {
  const { view, runs, open, onToggle } = props;
  return (
    <Card flush className="run-card" edgeTone={view.handed ? 'neutral' : undefined} data-run={view.item.id}>
      <Disclosure
        open={open}
        onOpenChange={onToggle}
        summary={<RunName id={view.item.id} title={view.plan?.title} />}
        end={<RunHead view={view} />}
        wrapEnd
      >
        <LaunchBox view={view} runs={runs} />
        <TestSections tests={view.tests} emptyText="No test in this run yet." />
      </Disclosure>
    </Card>
  );
};

export { RunCard };
