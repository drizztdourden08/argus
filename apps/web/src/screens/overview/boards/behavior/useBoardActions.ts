/* @layer renderer-app @kind logic */
import { useNavigate } from 'react-router';
import { postBoardOp } from '../../../../data/boards';
import type { Card } from '../../../../data/types';
import { useRecordPane } from '../../../../shell/records';
import { useMasterRun } from '../../dashboards/behavior/useMasterRun';
import type { SendDown } from '../../dashboards/dashboards.type';
import type { BoardActions } from '../boards.type';
import type { BoardRecord } from '../model';
import { idleCard } from './idle-card';

const useBoardActions = (record: BoardRecord, card: Card | null, send: SendDown): BoardActions => {
  const navigate = useNavigate();
  const pane = useRecordPane();
  const run = useMasterRun(card ?? idleCard(record));
  const url = card?.url ?? `/p/${record.key}/`;
  return {
    run,
    handlers: {
      open: () => void navigate(url),
      send: () => send.open(record.key),
      play: run.play,
      stop: run.stop,
      edit: () => pane.open(record.id, 'overview'),
      archive: () => void postBoardOp({ kind: 'archive', id: record.id }),
      unarchive: () => void postBoardOp({ kind: 'unarchive', id: record.id }),
    },
  };
};

export { useBoardActions };
