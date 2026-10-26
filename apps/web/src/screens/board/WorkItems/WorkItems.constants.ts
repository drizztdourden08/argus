/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';
import type { NoteKind } from '../owner-posts';

const ITEM_STATUSES = defineStatuses({
  todo: { label: 'To do', tone: 'neutral' },
  doing: { label: 'Doing', tone: 'info' },
  review: { label: 'Review', tone: 'warning' },
  done: { label: 'Done', tone: 'success' },
  dropped: { label: 'Dropped', tone: 'neutral' },
});

const NOTE_OPTIONS: { value: NoteKind; label: string }[] = [
  { value: 'decision', label: 'Decision' },
  { value: 'followup', label: 'Note' },
];

export { ITEM_STATUSES, NOTE_OPTIONS };
