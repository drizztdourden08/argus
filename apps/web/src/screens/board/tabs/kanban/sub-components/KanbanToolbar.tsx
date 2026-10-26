/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { KanbanToolbarProps } from '../kanban-props.type';
import { FilterRow } from './FilterRow';
import { FocusRow } from './FocusRow';
import { ViewRow } from './ViewRow';

const KanbanToolbar = (props: KanbanToolbarProps) => {
  const { state, view, threads, subjects, focus, choices } = props;
  return (
    <Flex direction="column" gap="sm" role="search" aria-label="Filters">
      <FilterRow state={state} threads={threads} subjects={subjects} />
      <FocusRow focus={focus} choices={choices} />
      <ViewRow state={state} view={view} />
    </Flex>
  );
};

export { KanbanToolbar };
