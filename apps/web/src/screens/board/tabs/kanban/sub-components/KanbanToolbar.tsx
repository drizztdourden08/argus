/* @layer renderer-app @kind component */
import { SegmentedControl, Text } from '@drizztdourden08/tessera';
import type { SortDirection } from '@drizztdourden08/tessera';
import { TabFilterBar, TabGroupSort, ToolToggle, useTabTools } from '../../../BoardTabsRow';
import { LAYOUT_OPTIONS } from '../../kanban-tree';
import type { KanbanLayout } from '../../kanban-tree';
import { DONE_TIP } from '../kanban.constants';
import type { KanbanToolbarProps } from '../kanban-props.type';
import { GROUP_OPTIONS, ORDER_OPTIONS, countsText, shownDirection } from '../model';
import type { ItemOrder } from '../model';
import { kanbanFields } from '../model/filter-fields';

const KanbanToolbar = (props: KanbanToolbarProps) => {
  const { boardKey, state, view, items, threads, agents, selection, layout } = props;
  const { prefs, filters, patchFilters, setGroup, setOrder, setDirection, setShowDone } = state;
  const sortBy = (order: ItemOrder, direction: SortDirection) => {
    if (order !== prefs.order) {
      setOrder(order);
      return;
    }
    setDirection(direction);
  };
  const layoutControl = (
    <SegmentedControl<KanbanLayout> selectedTone="neutral" aria-label="View" size="sm-plus" value={layout.layout} options={LAYOUT_OPTIONS} onChange={layout.setLayout} />
  );
  const groupSort = (
    <TabGroupSort
      group={{ options: GROUP_OPTIONS, value: prefs.group, empty: 'status', onChange: setGroup }}
      sort={{ options: ORDER_OPTIONS, order: prefs.order, direction: shownDirection(prefs.order, prefs.direction), onChange: sortBy }}
    />
  );
  useTabTools('kanban', [
    { id: 'view', priority: 4, node: layoutControl },
    { id: 'group-sort', priority: 5, node: groupSort },
    { id: 'done', priority: 3, node: <ToolToggle label={prefs.showDone ? 'Hide done' : 'Show done'} count={view.closed} pressed={prefs.showDone} onToggle={() => setShowDone(!prefs.showDone)} /> },
    { id: 'select', priority: 3, node: <ToolToggle label="Select" pressed={selection.active} onToggle={() => selection.setActive(!selection.active)} /> },
    { id: 'counts', priority: 1, node: <Text variant="caption" tone="muted" aria-live="polite" title={DONE_TIP}>{countsText(view)}</Text> },
  ]);
  return (
    <TabFilterBar tab="kanban" boardKey={boardKey} fields={kanbanFields(boardKey, items, threads, agents)} filters={filters} patchFilters={patchFilters} placeholder="Search id or title" />
  );
};

export { KanbanToolbar };
