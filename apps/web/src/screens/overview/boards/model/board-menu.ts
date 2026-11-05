/* @layer renderer-app @kind logic */
import type { BoardMenuItem, BoardRecord } from './board.type';

const playItems = (record: BoardRecord): BoardMenuItem[] => {
  if (record.masterRunning) return [{ id: 'stop', label: 'Stop master' }];
  return record.hasMaster ? [{ id: 'play', label: 'Play master' }] : [];
};

const liveItems = (record: BoardRecord): BoardMenuItem[] => [
  { id: 'send', label: 'Send to the lead' },
  ...playItems(record),
  { id: 'edit', label: 'Edit goal and note' },
  { id: 'archive', label: 'Archive', tone: 'danger' },
];

const boardMenu = (record: BoardRecord): BoardMenuItem[] => [
  { id: 'open', label: 'Open board' },
  ...(record.archived
    ? [{ id: 'edit', label: 'Edit goal and note' } as const, { id: 'unarchive', label: 'Unarchive' } as const]
    : liveItems(record)),
];

export { boardMenu };
