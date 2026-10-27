/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
import type { KanbanToolbarProps } from '../kanban-props.type';
import { FilterRow } from './FilterRow';
import { ViewRow } from './ViewRow';

const KanbanToolbar = (props: KanbanToolbarProps) => {
  const { state, view, threads, subjects } = props;
  return (
    <Card role="search" aria-label="Filters">
      <Flex direction="column" gap="sm">
        <FilterRow state={state} threads={threads} subjects={subjects} />
        <ViewRow state={state} view={view} />
      </Flex>
    </Card>
  );
};

export { KanbanToolbar };
