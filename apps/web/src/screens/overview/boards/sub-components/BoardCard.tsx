/* @layer renderer-app @kind component */
import { Pressable } from '@drizztdourden08/tessera';
import { RecordCard, useRecordPane } from '../../../../shell/records';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { attentionOf, cardTone } from '../model';
import { boardMenuGroups } from '../behavior/board-menu-groups';
import { useBoardActions } from '../behavior/useBoardActions';
import type { BoardCardProps } from '../boards.type';
import { BoardCardBody } from './BoardCardBody';
import { BoardCardFoot } from './BoardCardFoot';
import './BoardCard.css';

const BoardCard = (props: BoardCardProps) => {
  const { record, card, live, now, send, sync, onLog } = props;
  const { run, handlers } = useBoardActions(record, card, send);
  const pane = useRecordPane();
  const icon = <BoardIcon boardKey={record.key} icon={record.icon} size="sm" />;
  return (
    <RecordCard
      head={{
        kind: 'board',
        code: record.id,
        assignee: `lead:${record.key}`,
        boardKey: record.key,
        threads: [],
        menu: boardMenuGroups(record, record.started, handlers),
        menuLabel: `More actions for ${record.id}`,
        mark: record.started ? <Pressable onClick={handlers.open} aria-label={`Open the ${record.name} board`}>{icon}</Pressable> : icon,
      }}
      className="board-card"
      edge="border"
      fill
      edgeTone={record.started && card && !record.archived ? cardTone(attentionOf(card)) : undefined}
      attrs={{ 'data-swap-item': record.id, 'data-started': String(record.started) }}
      title={record.name}
      openLabel={`Open ${record.id}: ${record.name}`}
      onOpen={() => pane.open(record.id)}
      signals={<BoardCardBody record={record} card={card} live={live} now={now} run={run} onLog={onLog} onOpen={(tab) => pane.open(record.id, tab)} />}
      foot={record.started ? <BoardCardFoot record={record} card={card} now={now} send={send} sync={sync} /> : undefined}
    />
  );
};

export { BoardCard };
