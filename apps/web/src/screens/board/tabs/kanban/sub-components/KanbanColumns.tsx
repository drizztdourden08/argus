/* @layer renderer-app @kind component */
import { KanbanBoard } from '@drizztdourden08/tessera';
import type { KanbanMove } from '@drizztdourden08/tessera';
import { applyDrop } from '../behavior/apply-drop';
import { BOARD_LABEL } from '../kanban.constants';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { planDrop } from '../model';
import { ItemCard } from './ItemCard';

const KanbanColumns = (props: KanbanColumnsProps) => {
  const { view, state, threads, now, selection, ops, onOpen } = props;
  const { group, order } = state.prefs;
  const onMove = (move: KanbanMove) => {
    const card = view.cards.find((candidate) => candidate.id === move.cardId);
    if (card) applyDrop(planDrop({ columns: view.columns, move, group, order }), card.item, ops);
  };
  return (
    <KanbanBoard
      label={BOARD_LABEL}
      columns={view.columns}
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
          onToggle={selection.toggle}
          onOpen={onOpen}
        />
      )}
    />
  );
};

export { KanbanColumns };
