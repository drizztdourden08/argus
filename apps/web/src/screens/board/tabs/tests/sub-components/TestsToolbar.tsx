/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { useFocusActive } from '../../../BoardFocus';
import { ClauseFilterRow, TabFilters, useClauseFilters } from '../../../BoardTabsRow';
import { testsFields } from '../model/clause-fields';
import { FIND_TEXT } from '../tests.constants';
import type { TestsToolbarProps } from '../tests-props.type';
import { ViewRow } from './ViewRow';

const TestsToolbar = (props: TestsToolbarProps) => {
  const { boardKey, state, view, threads, subjects, plans, builds, handed } = props;
  const { filters, patchFilters } = state;
  const bar = useClauseFilters(testsFields({ threads, subjects, plans, builds, handed }), filters, patchFilters);
  const focused = useFocusActive(boardKey);
  const count = bar.count + (filters.query ? 1 : 0) + (focused ? 1 : 0);
  return (
    <>
      <TabFilters tab="tests" count={count}>
        <ClauseFilterRow boardKey={boardKey} search={filters.query} placeholder={FIND_TEXT} onSearch={(query) => patchFilters({ query })} bar={bar} />
      </TabFilters>
      <Card>
        <ViewRow state={state} view={view} />
      </Card>
    </>
  );
};

export { TestsToolbar };
