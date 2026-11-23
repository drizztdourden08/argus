/* @layer renderer-app @kind component */
import type { KanbanColumn } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { addPreset } from '../model';
import type { CardData, ItemGroup, KanbanView } from '../model';
import { ColumnAdd } from './ColumnAdd';
import { ColumnAddForm } from './ColumnAddForm';
import { LimitBadge } from './LimitBadge';
import { ParentHeader } from './ParentHeader';
import { QuietBadge } from './QuietBadge';

const quietIn = (cards: readonly CardData[]): number => cards.filter((card) => card.quietMs !== null).length;

const headerOf = (group: ItemGroup, parent: ItemRef | undefined): ReactNode => (group === 'parent' && parent ? <ParentHeader parent={parent} /> : undefined);

const badgeOf = (view: KanbanView, column: KanbanColumn<CardData>): ReactNode => {
  const limit = view.limits.get(column.id);
  const quiet = quietIn(column.cards);
  if (!limit && quiet === 0) return undefined;
  return (
    <>
      {limit ? <LimitBadge count={column.cards.length} limit={limit} /> : null}
      {quiet > 0 ? <QuietBadge count={quiet} /> : null}
    </>
  );
};

const columnDefs = (props: KanbanColumnsProps, source: readonly KanbanColumn<CardData>[]): KanbanColumn<CardData>[] => {
  const { view, state, threads, ops, add, focus } = props;
  const { group } = state.prefs;
  return source.map((column) => ({
    ...column,
    header: headerOf(group, view.parents.get(column.id)),
    badge: badgeOf(view, column),
    action: <ColumnAdd title={column.title} open={add.column === column.id} onToggle={() => add.toggle(column.id)} />,
    top:
      add.column === column.id ? (
        <ColumnAddForm
          preset={addPreset(group, view.targets.get(column.id) ?? column.id, focus)}
          group={group}
          threads={threads}
          items={view.parents}
          onCreate={(item) => void ops.create(item)}
          onClose={add.close}
        />
      ) : undefined,
  }));
};

export { columnDefs };
