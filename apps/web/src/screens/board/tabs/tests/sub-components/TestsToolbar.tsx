/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
import type { TestsToolbarProps } from '../tests-props.type';
import { FilterRow } from './FilterRow';
import { ViewRow } from './ViewRow';

const TestsToolbar = (props: TestsToolbarProps) => {
  const { state, view, threads, subjects, builds, handed } = props;
  return (
    <Card role="search" aria-label="Filters">
      <Flex direction="column" gap="sm">
        <FilterRow state={state} threads={threads} subjects={subjects} builds={builds} handed={handed} />
        <ViewRow state={state} view={view} />
      </Flex>
    </Card>
  );
};

export { TestsToolbar };
