/* @layer renderer-app @kind constants */
import type { ActionDef } from './records.type';

const EDIT: ActionDef = { id: 'open', label: 'Edit', group: 'open' };
const OPEN: ActionDef = { id: 'open', label: 'Open', group: 'open' };
const COPY: ActionDef = { id: 'copy', label: 'Copy id', group: 'soft' };
const ARCHIVE: ActionDef = { id: 'archive', label: 'Archive', group: 'soft' };
const UNARCHIVE: ActionDef = { id: 'unarchive', label: 'Unarchive', group: 'soft' };
const DELETE: ActionDef = { id: 'delete', label: 'Delete', group: 'soft', tone: 'danger' };

const WORK_ITEM: readonly ActionDef[] = [EDIT, ARCHIVE, UNARCHIVE, DELETE, COPY];
const PLAIN: readonly ActionDef[] = [OPEN, COPY];

const RECORD_ACTIONS: Readonly<Record<string, readonly ActionDef[]>> = {
  epic: WORK_ITEM,
  story: WORK_ITEM,
  task: WORK_ITEM,
  bug: WORK_ITEM,
  debt: WORK_ITEM,
  'test-plan': WORK_ITEM,
  'test-case': PLAIN,
  'test-run': PLAIN,
  build: [
    OPEN,
    { id: 'play', label: 'Play', group: 'act' },
    { id: 'copy-command', label: 'Copy command', group: 'act' },
    { id: 'link', label: 'Link to an item', group: 'act' },
    { id: 'retire', label: 'Retire', group: 'soft', tone: 'danger' },
    COPY,
  ],
  worktree: [
    OPEN,
    { id: 'keep', label: 'Keep', group: 'act' },
    { id: 'link', label: 'Link to an item', group: 'act' },
    { id: 'release', label: 'Release', group: 'act' },
    { id: 'remove', label: 'Remove', group: 'soft', tone: 'danger' },
    COPY,
  ],
  agent: [
    OPEN,
    { id: 'stop', label: 'Stop', group: 'act' },
    { id: 'set-item', label: 'Set item', group: 'act' },
    ARCHIVE,
    UNARCHIVE,
    COPY,
  ],
  change: PLAIN,
  release: PLAIN,
  dependency: [OPEN, { id: 'cancel', label: 'Cancel', group: 'soft', tone: 'danger' }, COPY],
  run: [OPEN, { id: 'stop', label: 'Stop', group: 'act' }, COPY],
  check: PLAIN,
  'removal-request': PLAIN,
  question: PLAIN,
  decision: PLAIN,
  todo: [OPEN, { id: 'done', label: 'Mark done', group: 'act' }, COPY],
  note: PLAIN,
  notice: PLAIN,
  followup: PLAIN,
  message: PLAIN,
};

const MENU_GROUP_IDS = { open: 'edit', act: 'act', soft: 'soft' } as const;

export { MENU_GROUP_IDS, RECORD_ACTIONS };
