/* @layer renderer-app @kind constants */
import type { NoteRule } from './cell-note.type';

const NOTE_RULES: Readonly<Record<string, NoteRule>> = {
  copy_to: { test: 'target-missing', note: 'will be created' },
  create_on: { test: 'target-missing', note: 'will be created' },
  delete_on: { test: 'target-present', note: 'will be deleted' },
  take: { test: 'other-present', note: 'will be updated' },
  folder_of: { test: 'other-present', note: 'will change folder' },
  archive_all: { test: 'active', note: 'will be archived' },
  unarchive_all: { test: 'archived', note: 'will be unarchived' },
  link_all: { test: 'not-linked', note: 'will be linked' },
  unlink_all: { test: 'linked', note: 'will be unlinked' },
};

export { NOTE_RULES };
