/* @layer renderer-app @kind component */
import './RunCard.css';
import { useRunMenu } from '../behavior/useRunMenu';
import type { RunCardProps } from '../tests-props.type';
import { RunActions } from './RunActions';
import { RunChips } from './RunChips';
import { RunFrame } from './RunFrame';
import { TestSections } from './TestSections';

const RunCard = (props: RunCardProps) => {
  const { view, runs, open, onToggle } = props;
  const menu = useRunMenu(view, runs);
  return (
    <RunFrame
      view={view}
      open={open}
      className="run-card"
      onToggle={onToggle}
      menu={menu}
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
