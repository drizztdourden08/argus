/* @layer renderer-app @kind constants */
import type { NoteFilters } from './model';

const NONE_YET = 'No notification yet. Threads ping you here when you are needed.';

const NO_FILTERS: NoteFilters = { level: 'all', project: '' };

const PART_LABEL = { toast: 'Toast', sound: 'Sound' } as const;

export { NO_FILTERS, NONE_YET, PART_LABEL };
