/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import type { Build, Test } from '../../../../../data/types';
import { toRunCount } from '../model';

const useTestsBadge = (boardKey: string): number => {
  const tests = useCollection<Test>(boardKey, 'tests');
  const builds = useCollection<Build>(boardKey, 'builds');
  return useMemo(() => toRunCount(tests.docs, builds.docs), [tests.docs, builds.docs]);
};

export { useTestsBadge };
