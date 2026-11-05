/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import type { Test } from '../../../../../data/types';
import { toRunCount } from '../model';

const useTestsBadge = (boardKey: string): number => {
  const tests = useCollection<Test>(boardKey, 'tests');
  return useMemo(() => toRunCount(tests.docs), [tests.docs]);
};

export { useTestsBadge };
