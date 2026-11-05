/* @layer renderer-app @kind component */
import type { KanbanColumn } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../../data/kanban';
import type { KanbanColumnsProps } from '../kanban-props.type';
import { addPreset } from '../model';
import type { CardData, ItemGroup } from '../model';
import { ColumnAdd } from './ColumnAdd';
import { ColumnAddForm } from './ColumnAddForm';
import { ParentHeader } from './ParentHeader';
import { QuietBadge } from './QuietBadge';

const quietIn = (cards: readonly CardData[]): number => cards.filter((card) => card.quietMs !== null).length;

const headerOf = (group: ItemGroup, parent: ItemRef | undefined): ReactNode => (group === 'parent' && parent ? <ParentHeader parent={parent} withCode /> : undefined);

const columnDefs = (props: KanbanColumnsProps): KanbanColumn<CardData>[] => {
  const { view, state, threads, ops, add, focus } = props;
  const { group } = state.prefs;
  return view.columns.map((column) => ({
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
          items={view.parents}
          onCreate={(item) => void ops.create(item)}
          onClose={add.close}
        />
      ) : undefined,
  }));
};

export { columnDefs };
