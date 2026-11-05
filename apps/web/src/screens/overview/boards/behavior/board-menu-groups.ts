/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import { NEEDS_BOARD } from '../boards.constants';
import type { BoardHandlers } from '../boards.type';
import { boardMenu } from '../model';
import type { BoardRecord } from '../model';

const boardMenuGroups = (record: BoardRecord, started: boolean, handlers: BoardHandlers): MenuGroup[] => [
  {
    id: 'board',
    label: record.id,
    items: boardMenu(record).map((entry) => ({
      id: entry.id,
      label: entry.label,
      tone: entry.tone,
      disabled: !started && NEEDS_BOARD.includes(entry.id),
      onSelect: handlers[entry.id],
    })),
  },
];

export { boardMenuGroups };
