/* @layer renderer-app @kind constants */
import type { ReviewEdit } from '../../../../data/types';
import type { ReviewFilter, Side } from './review.type';

const REVIEW_PATH = '/api/accounts/review';
const APPLY_PATH = '/api/accounts/review/apply';
const CLEANUP_PATH = '/api/accounts/cleanup';

const NO_FOLDER_ID = 'folder:';
const NO_FOLDER = 'No folder';
const NO_FOLDER_VALUE = '__none__';
const DEFAULT_DAYS = 30;
const DAY_MS = 86_400_000;

const SCAN_TEXT = 'Scanning both accounts...';
const READ_ONLY_TEXT = 'This is a preview: changes are not written.';
const BUSY_TEXT = 'A sync or switch is already running.';
const APPLY_NOTE = 'Desktop closes while the changes are written and opens again after.';
const APPLY_STARTED = 'Changes sent. Desktop is closing.';
const EMPTY_REVIEW = 'No chats or folders found in the accounts.';
const NOTHING_TEXT = 'Nothing matches this filter.';
const CLEANUP_NONE = 'No archived chats are that old.';
const CLEANUP_NOTE = 'Archived chats not touched for this many days.';

const FILTER_OPTIONS: readonly { value: ReviewFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'differences', label: 'Differences' },
  { value: 'archived', label: 'Archived' },
];

const EDIT_LABEL: Readonly<Record<ReviewEdit['op'], string>> = {
  move_chat: 'move',
  archive: 'archive',
  unarchive: 'unarchive',
  delete_chat: 'chat delete',
  rename_folder: 'folder rename',
  delete_folder: 'folder delete',
  create_folder: 'new folder',
};

const NO_ROWS: never[] = [];

const CHOICE_SKIP = 'skip';

const SIDE_LOOK: Readonly<Record<Side, { label: string; icon: 'arrow-left' | 'arrow-right' | 'pause' }>> = {
  left: { label: 'Take left', icon: 'arrow-left' },
  right: { label: 'Take right', icon: 'arrow-right' },
  skip: { label: 'Skip', icon: 'pause' },
};

export {
  APPLY_NOTE, APPLY_PATH, APPLY_STARTED, BUSY_TEXT, CHOICE_SKIP, CLEANUP_NONE, CLEANUP_NOTE, CLEANUP_PATH, DAY_MS, DEFAULT_DAYS, EDIT_LABEL, EMPTY_REVIEW,
  FILTER_OPTIONS, NO_FOLDER, NO_FOLDER_VALUE, NO_FOLDER_ID, NO_ROWS, NOTHING_TEXT, READ_ONLY_TEXT, REVIEW_PATH, SCAN_TEXT, SIDE_LOOK,
};
