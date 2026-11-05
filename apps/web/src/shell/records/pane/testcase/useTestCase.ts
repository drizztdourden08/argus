/* @layer renderer-app @kind logic */
import { useCollection } from '../../../../data/react';
import type { Test, TestItem } from '../../../../data/types';
import { useTestOps } from '../../../../screens/board/tabs/tests/behavior/useTestOps';
import type { RecordData } from '../pane.type';
import type { TestCaseLoad } from './testcase.type';

const useTestCase = (data: RecordData): TestCaseLoad => {
  const { boardKey, target } = data;
  const tests = useCollection<Test>(boardKey, 'tests');
  const items = useCollection<TestItem>(boardKey, 'items');
  const ops = useTestOps(boardKey);
  const test = tests.docs.find((doc) => doc.id === target.id) ?? null;
  const item = items.docs.find((doc) => doc.id === test?.item) ?? null;
  return { test, item, items: items.docs, ops };
};

export { useTestCase };
