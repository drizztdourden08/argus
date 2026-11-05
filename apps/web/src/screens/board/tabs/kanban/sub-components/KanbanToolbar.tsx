/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { TabFilterSet, TabViewMenu } from '../../../BoardTabsRow';
import { kanbanFields } from '../model/clause-fields';
import type { KanbanToolbarProps } from '../kanban-props.type';
import { GROUP_OPTIONS, ORDER_OPTIONS, shownDirection } from '../model';
import { ViewRow } from './ViewRow';

const KanbanToolbar = (props: KanbanToolbarProps) => {
  const { boardKey, state, view, items, threads, selection, layout } = props;
  return (
    <>
      <TabViewMenu groups={GROUP_OPTIONS} orders={ORDER_OPTIONS} state={state} shown={shownDirection} />
      <TabFilterSet
        tab="kanban"
        boardKey={boardKey}
        fields={kanbanFields(boardKey, items, threads)}
        filters={state.filters}
        patchFilters={state.patchFilters}
        placeholder="Search id or title"
      />
      <Card container>
        <ViewRow state={state} view={view} selection={selection} layout={layout} />
      </Card>
    </>
  );
};

export { KanbanToolbar };
