/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { useFocusActive } from '../../../BoardFocus';
import { ClauseFilterRow, TabFilters, useClauseFilters } from '../../../BoardTabsRow';
import { kanbanFields } from '../model/clause-fields';
import type { KanbanToolbarProps } from '../kanban-props.type';
import { ViewRow } from './ViewRow';

const KanbanToolbar = (props: KanbanToolbarProps) => {
  const { boardKey, state, view, items, threads, selection, layout } = props;
  const { filters, patchFilters } = state;
  const bar = useClauseFilters(kanbanFields(boardKey, items, threads), filters, patchFilters);
  const focused = useFocusActive(boardKey);
  const count = bar.count + (filters.query ? 1 : 0) + (focused ? 1 : 0);
  return (
    <>
      <TabFilters tab="kanban" count={count}>
        <ClauseFilterRow boardKey={boardKey} search={filters.query} placeholder="Search id or title" onSearch={(query) => patchFilters({ query })} bar={bar} />
      </TabFilters>
      <Card container>
        <ViewRow state={state} view={view} selection={selection} layout={layout} />
      </Card>
    </>
  );
};

export { KanbanToolbar };
