/* @layer renderer-app @kind component */
import { KanbanBoard, TesseraProvider } from '@drizztdourden08/tessera';
import type { KanbanMove } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import { itemCode } from '../../../../../shell';
import { applyDrop } from '../behavior/apply-drop';
import { leadTone } from '../attention';
import { BOARD_LABEL, EMPTY_COLUMN_STRINGS } from '../kanban.constants';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { planDrop } from '../model';
import type { CardData } from '../model';
import { columnDefs } from './column-defs';
import { ItemCard } from './ItemCard';
import { ParentHeader } from './ParentHeader';

const frameHeader = (parents: ReadonlyMap<string, ItemRef>, id: string): ReactNode => {
  const parent = parents.get(id);
  return parent ? <ParentHeader parent={parent} withCode={false} /> : undefined;
};

const KanbanColumns = (props: KanbanColumnsProps) => {
  const { boardKey, view, state, threads, now, selection, ops, onOpen } = props;
  const { group, order } = state.prefs;
  const ids = view.columns.flatMap((column) => column.cards.map((card) => card.id));
  const onMove = (move: KanbanMove) => {
    const card = view.cards.find((candidate) => candidate.id === move.cardId);
    if (card) applyDrop(planDrop({ columns: view.columns, move, group, order }), card.item, ops);
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
      columns={columns}
      cardLabel={(card) => `${itemCode(card.item)} ${card.item.title}`}
      cardEdgeTone={(card) => leadTone(card.attention)}
      onMove={onMove}
      onCardOpen={onCardOpen}
      renderGroupHeader={(groupId) => frameHeader(view.parents, groupId)}
      renderCard={(card) => (
        <ItemCard
          boardKey={boardKey}
          card={card}
          group={group}
          threads={threads}
          now={now}
          selected={selection.has(card.id)}
          selecting={selection.active}
          ops={ops}
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
