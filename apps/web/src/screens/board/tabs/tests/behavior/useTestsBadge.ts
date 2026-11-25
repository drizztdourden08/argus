/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import type { Test } from '../../../../../data/types';
import { toRunCount } from '../model';

interface TestsCount {
  total: number;
  toRun: number;
}

const useTestsBadge = (boardKey: string): TestsCount => {
  const tests = useCollection<Test>(boardKey, 'tests');
  return useMemo(() => ({ total: tests.docs.filter((test) => !test.deletedAt).length, toRun: toRunCount(tests.docs) }), [tests.docs]);
};

export { useTestsBadge };
