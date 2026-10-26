/* @layer renderer-app @kind constants */
import type { NoteFilters } from './model';

const NONE_YET = 'No notification yet. Threads ping you here when you are needed.';

const NO_FILTERS: NoteFilters = { level: 'all', project: '' };

const ALL_PROJECTS = '*';

const PART_LABEL = { toast: 'Toast', sound: 'Sound' } as const;

export { ALL_PROJECTS, NO_FILTERS, NONE_YET, PART_LABEL };
