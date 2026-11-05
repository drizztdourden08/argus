/* @layer renderer-app @kind component */
import { KanbanBoard, TesseraProvider } from '@drizztdourden08/tessera';
import type { KanbanGroupFold, KanbanMove } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import { itemCode } from '../../../../../shell';
import { applyDrop } from '../behavior/apply-drop';
import { leadTone } from '../attention';
import { BOARD_LABEL, EMPTY_COLUMN_STRINGS } from '../kanban.constants';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { planDrop } from '../model';
import { columnDefs } from './ColumnDefs';
import { ItemCard } from './ItemCard';
import { ParentHeader } from './ParentHeader';

const frameHeader = (parents: ReadonlyMap<string, ItemRef>, id: string, fold: KanbanGroupFold): ReactNode => {
  const parent = parents.get(id);
  return parent ? <ParentHeader parent={parent} withCode={false} fold={fold} /> : undefined;
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
      stack
      cardFrame="none"
      columns={columns}
      cardLabel={(card) => `${itemCode(card.item)} ${card.item.title}`}
      cardEdgeTone={(card) => leadTone(card.attention)}
      onMove={onMove}
      onCardOpen={onCardOpen}
      renderGroupHeader={(groupId, _column, fold) => frameHeader(view.parents, groupId, fold)}
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
