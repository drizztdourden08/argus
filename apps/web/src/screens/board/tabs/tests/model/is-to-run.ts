/* @layer renderer-app @kind logic */
import type { Test } from '../../../../../data/types';

const isToRun = (test: Test): boolean => test.status === 'pending' && !test.archivedAt;

export { isToRun };
