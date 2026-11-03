/* @layer renderer-app @kind logic */
import { useCollection, useDoc } from '../../../../data/react';
import type { BoardMeta, Test, TestItem } from '../../../../data/types';
import { useTestOps } from '../../../../screens/board/tabs/tests/behavior/useTestOps';
import type { RecordData } from '../pane.type';
import type { TestCaseLoad } from './testcase.type';

const useTestCase = (data: RecordData): TestCaseLoad => {
  const { boardKey, target } = data;
  const tests = useCollection<Test>(boardKey, 'tests');
  const items = useCollection<TestItem>(boardKey, 'items');
  const meta = useDoc<BoardMeta>(boardKey, 'meta', 'board');
  const ops = useTestOps(boardKey);
  const test = tests.docs.find((doc) => doc.id === target.id) ?? null;
  const item = items.docs.find((doc) => doc.id === test?.item) ?? null;
  const handed = Boolean(test?.build) && test?.build === (meta.doc?.handedBuild ?? null);
  return { test, item, items: items.docs, handed, ops };
};

export { useTestCase };
