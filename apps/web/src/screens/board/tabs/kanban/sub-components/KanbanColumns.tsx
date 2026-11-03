/* @layer renderer-app @kind component */
import { KanbanBoard, TesseraProvider } from '@drizztdourden08/tessera';
import type { KanbanColumn, KanbanMove } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import { applyDrop } from '../behavior/apply-drop';
import { leadTone } from '../attention';
import { BOARD_LABEL, EMPTY_COLUMN_STRINGS } from '../kanban.constants';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { addPreset, planDrop } from '../model';
import type { CardData, ItemGroup } from '../model';
import { ColumnAdd } from './ColumnAdd';
import { ColumnAddForm } from './ColumnAddForm';
import { ItemCard } from './ItemCard';
import { ParentHeader } from './ParentHeader';
import { QuietBadge } from './QuietBadge';

const quietIn = (cards: readonly CardData[]): number => cards.filter((card) => card.quietMs !== null).length;

const headerOf = (group: ItemGroup, parent: ItemRef | undefined): ReactNode => (group === 'parent' && parent ? <ParentHeader parent={parent} /> : undefined);

const KanbanColumns = (props: KanbanColumnsProps) => {
  const { boardKey, view, state, threads, now, selection, ops, add, focus, onOpen } = props;
  const { group, order } = state.prefs;
  const ids = view.columns.flatMap((column) => column.cards.map((card) => card.id));
  const onMove = (move: KanbanMove) => {
    const card = view.cards.find((candidate) => candidate.id === move.cardId);
    if (card) applyDrop(planDrop({ columns: view.columns, move, group, order }), card.item, ops);
  };
  const columns: KanbanColumn<CardData>[] = view.columns.map((column) => ({
    ...column,
    header: headerOf(group, view.parents.get(column.id)),
    badge: quietIn(column.cards) > 0 ? <QuietBadge count={quietIn(column.cards)} /> : undefined,
    action: <ColumnAdd title={column.title} open={add.column === column.id} onToggle={() => add.toggle(column.id)} />,
    top:
      add.column === column.id ? (
        <ColumnAddForm
          preset={addPreset(group, column.id, focus)}
          group={group}
          threads={threads}
          onCreate={(item) => void ops.create(item)}
          onClose={add.close}
        />
      ) : undefined,
  }));
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
      cardLabel={(card) => `${card.id} ${card.item.title}`}
      cardEdgeTone={(card) => leadTone(card.attention)}
      onMove={onMove}
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
