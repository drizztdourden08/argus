/* @layer renderer-app @kind component */
import './RunCard.css';
import type { RunCardProps } from '../tests-props.type';
import { RunActions } from './RunActions';
import { RunChips } from './RunChips';
import { RunFrame } from './RunFrame';
import { RunMenu } from './RunMenu';
import { TestSections } from './TestSections';

const RunCard = (props: RunCardProps) => {
  const { view, runs, open, onToggle } = props;
  return (
    <RunFrame
      view={view}
      open={open}
      className="run-card"
      onToggle={onToggle}
      edgeTone={view.handed ? 'neutral' : undefined}
      menu={<RunMenu view={view} runs={runs} />}
      signals={(
        <>
          <RunChips view={view} />
          <RunActions view={view} runs={runs} />
        </>
      )}
    >
      <TestSections tests={view.tests} emptyText="No test in this run yet." />
    </RunFrame>
  );
};

export { RunCard };
