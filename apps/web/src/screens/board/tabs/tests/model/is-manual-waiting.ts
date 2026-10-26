/* @layer renderer-app @kind logic */
import type { TestDoc } from './model.type';

const isManualWaiting = (test: TestDoc): boolean => test.mode === 'manual' && test.status === 'pending' && !test.archivedAt;

export { isManualWaiting };
