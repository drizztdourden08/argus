/* @layer renderer-app @kind logic */
import type { WithId } from '../../../../../data/react';
import type { Test } from '../../../../../data/types';
import { FAILING_STATUS } from '../board.constants';

const failingTests = (tests: readonly WithId<Test>[]): WithId<Test>[] => tests.filter((test) => test.status === FAILING_STATUS && !test.deletedAt && !test.archivedAt);

export { failingTests };
