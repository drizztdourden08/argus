/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import type { TestsShared } from '../tests.type';
import { TestsContext } from './tests-context';

const useTestsShared = (): TestsShared => {
  const shared = useContext(TestsContext);
  if (!shared) throw new Error('The Tests tab pieces need the Tests tab around them.');
  return shared;
};

export { useTestsShared };
