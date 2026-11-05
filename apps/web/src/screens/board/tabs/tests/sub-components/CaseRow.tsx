/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { RecordRow } from '../../../../../shell';
import { rowKeys } from '../behavior/row-keys';
import { testMenuGroups } from '../behavior/test-menu-groups';
import { useOutcome } from '../behavior/useOutcome';
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import { caseAssignee, caseChips, isToRun } from '../model';
import { CASE_KIND } from '../tests.constants';
import type { TestPartProps } from '../tests-props.type';
import { DoneSummary } from './DoneSummary';
import { OutcomeButtons } from './OutcomeButtons';
import { ResultComposer } from './ResultComposer';

const CaseRow = (props: TestPartProps) => {
  const { test } = props;
  const shared = useTestsShared();
  const { boardKey, threads, group, conflicted, onOpen } = shared;
  const controls = useOutcome(boardKey, test.id);
  const actions = useTestActions(boardKey);
  const toRun = isToRun(test);
  return (
    <Box data-swap-item data-test={test.id} data-to-run={toRun ? '' : undefined} onKeyDown={rowKeys(toRun, controls)}>
      <RecordRow
        head={{
          kind: CASE_KIND,
          code: test.id,
          assignee: caseAssignee(test),
          boardKey,
          threads,
          menu: testMenuGroups({ test, shared, rerun: () => void actions.rerun(test.id) }),
          menuLabel: `More actions for ${test.id}`,
        }}
        title={test.title}
        onOpen={() => onOpen(test.id)}
        chips={caseChips(test, group, conflicted)}
        trailing={toRun ? <OutcomeButtons testId={test.id} active={controls.open} busy={controls.busy} onChoose={controls.choose} /> : <DoneSummary test={test} />}
      />
      {controls.open && <ResultComposer testId={test.id} outcome={controls.open} controls={controls} />}
    </Box>
  );
};

export { CaseRow };
