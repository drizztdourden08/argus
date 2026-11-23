/* @layer renderer-app @kind constants */
import type { KanbanColumnLimit } from '../../../../../data/types';
import { LIMIT_MODES } from '../model';

const FIRST_LIMIT: KanbanColumnLimit = { max: 3, mode: 'warn' };

const MODE_OPTIONS = LIMIT_MODES.map((mode) => ({ value: mode, label: mode }));

export { FIRST_LIMIT, MODE_OPTIONS };
