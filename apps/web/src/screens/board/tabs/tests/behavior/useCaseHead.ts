/* @layer renderer-app @kind logic */
import { caseAssignee, caseChips, caseTime, isToRun } from '../model';
import type { TestDoc } from '../model';
import { CASE_KIND } from '../tests.constants';
import type { CaseHead } from '../tests.type';
import { testMenuGroups } from './test-menu-groups';
import { useOutcome } from './useOutcome';
import { useTestActions } from './useTestActions';
import { useTestsShared } from './useTestsShared';

const useCaseHead = (test: TestDoc): CaseHead => {
  const shared = useTestsShared();
  const { boardKey, threads, group, conflicted, onOpen } = shared;
  const controls = useOutcome(boardKey, test.id);
  const actions = useTestActions(boardKey);
  return {
    head: {
      kind: CASE_KIND,
      code: test.id,
      assignee: caseAssignee(test),
      boardKey,
      threads,
      menu: testMenuGroups({ test, shared, rerun: () => void actions.rerun(test.id) }),
      menuLabel: `More actions for ${test.id}`,
    },
    title: test.title,
    time: caseTime(test),
    chips: caseChips(test, group, conflicted),
    toRun: isToRun(test),
    controls,
    open: () => onOpen(test.id),
  };
};

export { useCaseHead };
