/* @layer renderer-app @kind component */
import type { Entry } from '../../../../data/kanban';
import { useCollection } from '../../../../data/react';
import type { Test } from '../../../../data/types';
import { useTestNotices } from '../../../../screens/board/tabs/tests/behavior/useTestNotices';
import { TestNotices } from '../../../../screens/board/tabs/tests/sub-components/TestNotices';
import type { CaseConflictsProps } from './testcase.type';

const CaseConflicts = (props: CaseConflictsProps) => {
  const { testId, boardKey } = props;
  const entries = useCollection<Entry>(boardKey, 'entries');
  const tests = useCollection<Test>(boardKey, 'tests');
  const ops = useTestNotices(boardKey, entries.docs, tests.docs);
  const mine = ops.notices.filter((notice) => notice.kind === 'conflict' && notice.op.target?.id === testId);
  return <TestNotices label={`Conflicts on ${testId}`} notices={mine} ops={ops} />;
};

export { CaseConflicts };
