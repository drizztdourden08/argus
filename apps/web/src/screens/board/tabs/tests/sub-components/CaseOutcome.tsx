/* @layer renderer-app @kind component */
import type { CaseOutcomeProps } from '../tests-head-props.type';
import { DoneSummary } from './DoneSummary';
import { OutcomeButtons } from './OutcomeButtons';

const CaseOutcome = (props: CaseOutcomeProps) => {
  const { test, view } = props;
  const { controls } = view;
  if (!view.toRun) return <DoneSummary test={test} />;
  return <OutcomeButtons testId={test.id} active={controls.open} busy={controls.busy} onChoose={controls.choose} />;
};

export { CaseOutcome };
