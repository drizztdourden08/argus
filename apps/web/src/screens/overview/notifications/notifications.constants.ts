/* @layer renderer-app @kind constants */
import type { ListItemHeader, ListItemShape } from '@drizztdourden08/tessera';
import type { NoteFilters } from './model';

const NONE_YET = 'No notification yet. Threads ping you here when you are needed.';

const NO_FILTERS: NoteFilters = { level: 'all', project: '' };

const NOTES_HEADER: ListItemHeader = {
  icon: 'Level',
  main: 'Notification',
  columns: [{ label: 'When', align: 'end' }, { label: 'Project' }],
};

const NOTES_SHAPE: ListItemShape = { icon: true, columns: 2, action: false };

const PART_LABEL = { toast: 'Toast', sound: 'Sound' } as const;

export { NO_FILTERS, NONE_YET, NOTES_HEADER, NOTES_SHAPE, PART_LABEL };
