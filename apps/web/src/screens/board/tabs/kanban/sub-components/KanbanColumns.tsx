/* @layer renderer-app @kind component */
import { KanbanBoard } from '@drizztdourden08/tessera';
import type { KanbanColumn, KanbanMove } from '@drizztdourden08/tessera';
import { applyDrop } from '../behavior/apply-drop';
import { BOARD_LABEL } from '../kanban.constants';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { addPreset, planDrop } from '../model';
import type { CardData } from '../model';
import { ColumnAdd } from './ColumnAdd';
import { ColumnAddForm } from './ColumnAddForm';
import { ItemCard } from './ItemCard';
import { QuietBadge } from './QuietBadge';

const quietIn = (cards: readonly CardData[]): number => cards.filter((card) => card.quietMs !== null).length;

const KanbanColumns = (props: KanbanColumnsProps) => {
  const { view, state, threads, now, selection, ops, add, focus, onOpen } = props;
  const { group, order } = state.prefs;
  const ids = view.columns.flatMap((column) => column.cards.map((card) => card.id));
  const onMove = (move: KanbanMove) => {
    const card = view.cards.find((candidate) => candidate.id === move.cardId);
    if (card) applyDrop(planDrop({ columns: view.columns, move, group, order }), card.item, ops);
  };
  const columns: KanbanColumn<CardData>[] = view.columns.map((column) => ({
    ...column,
    badge: quietIn(column.cards) > 0 ? <QuietBadge count={quietIn(column.cards)} /> : undefined,
    action: (
      <ColumnAdd title={column.title} open={add.column === column.id} onToggle={() => add.toggle(column.id)} onClose={add.close}>
        <ColumnAddForm
          preset={addPreset(group, column.id, focus)}
          group={group}
          threads={threads}
          onCreate={(item) => void ops.create(item)}
          onClose={add.close}
        />
      </ColumnAdd>
    ),
  }));
  return (
    <KanbanBoard
      label={BOARD_LABEL}
      columns={columns}
      cardLabel={(card) => `${card.id} ${card.item.title}`}
      onMove={onMove}
      renderCard={(card) => (
        <ItemCard
          card={card}
          group={group}
          threads={threads}
          now={now}
          selected={selection.has(card.id)}
          ops={ops}
          onToggle={(id) => selection.toggle(id, ids)}
          onMark={selection.mark}
          onOpen={onOpen}
        />
      )}
    />
  );
};

export { KanbanColumns };
