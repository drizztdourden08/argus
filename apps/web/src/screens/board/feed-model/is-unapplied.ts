/* @layer renderer-app @kind logic */
import type { Entry } from '../../../data/types';

const isUnapplied = (entry: Entry): boolean => !entry.appliedAt && (!entry.status || entry.status === 'new');

export { isUnapplied };
