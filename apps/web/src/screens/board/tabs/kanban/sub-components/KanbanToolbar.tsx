/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
import type { KanbanToolbarProps } from '../kanban-props.type';
import { FilterRow } from './FilterRow';
import { ViewRow } from './ViewRow';

const KanbanToolbar = (props: KanbanToolbarProps) => {
  const { boardKey, state, view, items, threads, selection, layout } = props;
  return (
    <Card container role="search" aria-label="Filters">
      <Flex direction="column" gap="sm">
        <FilterRow boardKey={boardKey} state={state} items={items} threads={threads} />
        <ViewRow state={state} view={view} selection={selection} layout={layout} />
      </Flex>
    </Card>
  );
};

export { KanbanToolbar };
