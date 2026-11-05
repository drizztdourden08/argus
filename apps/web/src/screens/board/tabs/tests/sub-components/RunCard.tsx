/* @layer renderer-app @kind component */
import './RunCard.css';
import type { RunCardProps } from '../tests-props.type';
import { RunActions } from './RunActions';
import { RunFrame } from './RunFrame';
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
      attrs={{ 'data-swap-item': '', 'data-run': view.item.id }}
      content={<RunActions view={view} runs={runs} />}
    >
      <TestSections tests={view.tests} emptyText="No test in this run yet." />
    </RunFrame>
  );
};

export { RunCard };
