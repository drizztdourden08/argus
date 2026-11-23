/* @layer renderer-app @kind component */
import { KanbanBoard, TesseraProvider } from '@drizztdourden08/tessera';
import type { KanbanColumn, KanbanGroupFold, KanbanMove } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import { itemCode } from '../../../../../shell';
import { applyDrop } from '../behavior/apply-drop';
import { cardMenuGroups } from '../behavior/card-menu-groups';
import { leadTone } from '../attention';
import { BOARD_LABEL, EMPTY_COLUMN_STRINGS } from '../kanban.constants';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { planDrop } from '../model';
import type { CardData } from '../model';
import { columnDefs } from './ColumnDefs';
import { EpicHeader } from './EpicHeader';
import { ItemCard } from './ItemCard';

const frameHeader = (props: KanbanColumnsProps, id: string, column: KanbanColumn<CardData>, fold: KanbanGroupFold): ReactNode => {
  const { view, threads, ops, onOpen } = props;
  const parent = view.parents.get(id);
  if (!parent) return undefined;
  const count = column.cards.filter((card) => card.group === id).length;
  return <EpicHeader parent={parent} count={count} menu={cardMenuGroups({ item: parent, threads, ops, onOpen })} fold={fold} onOpen={onOpen} />;
};

const KanbanColumns = (props: KanbanColumnsProps) => {
  const { boardKey, view, state, threads, now, selection, ops, onOpen } = props;
  const { group, order, direction } = state.prefs;
  const ids = view.columns.flatMap((column) => column.cards.map((card) => card.id));
  const onMove = (move: KanbanMove) => {
    const card = view.cards.find((candidate) => candidate.id === move.cardId);
    if (card) applyDrop(planDrop({ columns: view.columns, move, group, order, direction }), card.item, ops);
  };
  const onCardOpen = (cardId: string) => {
    const card = view.cards.find((candidate) => candidate.id === cardId);
    if (card) onOpen(itemCode(card.item));
  };
  const columns = columnDefs(props);
  return (
    <TesseraProvider overrides={EMPTY_COLUMN_STRINGS}>
    <KanbanBoard
      label={BOARD_LABEL}
      columnWidth="fill"
      columnHeight="fit"
      grip={false}
      foldable={false}
      foldedGroups={state.foldedGroups}
      onFoldedGroupsChange={state.setFoldedGroups}
      stack
      cardFrame="none"
      columns={columns}
      cardLabel={(card) => `${itemCode(card.item)} ${card.item.title}`}
      cardEdgeTone={(card) => leadTone(card.attention)}
      onMove={onMove}
      onCardOpen={onCardOpen}
      renderGroupHeader={(groupId, column, fold) => frameHeader(props, groupId, column, fold)}
      renderCard={(card, frame) => (
        <ItemCard
          boardKey={boardKey}
          card={card}
          group={group}
          threads={threads}
          now={now}
          selected={selection.has(card.id)}
          selecting={selection.active}
          ops={ops}
          edgeTone={frame.edgeTone}
          compact={state.compact}
          onToggle={(id) => selection.toggle(id, ids)}
          onMark={selection.mark}
          onOpen={onOpen}
        />
      )}
    />
    </TesseraProvider>
  );
};

export { KanbanColumns };
