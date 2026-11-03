/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { Test, TestItem } from '../../../../data/types';
import type { TestOps } from '../../../../screens/board/tabs/tests/tests.type';

interface TestCaseLoad {
  test: WithId<Test> | null;
  item: WithId<TestItem> | null;
  items: WithId<TestItem>[];
  handed: boolean;
  ops: TestOps;
}

interface CaseConflictsProps {
  testId: string;
  boardKey: string;
}

interface CaseResultProps {
  test: WithId<Test>;
  boardKey: string;
}

export type { CaseConflictsProps, CaseResultProps, TestCaseLoad };
