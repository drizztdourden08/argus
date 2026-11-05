/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { TabFilterSet, TabViewMenu } from '../../../BoardTabsRow';
import { testsFields } from '../model/clause-fields';
import { GROUP_OPTIONS, ORDER_OPTIONS, shownDirection } from '../model';
import { FIND_TEXT } from '../tests.constants';
import type { TestsToolbarProps } from '../tests-props.type';
import { ViewRow } from './ViewRow';

const TestsToolbar = (props: TestsToolbarProps) => {
  const { boardKey, state, view, threads, subjects, plans } = props;
  return (
    <>
      <TabViewMenu groups={GROUP_OPTIONS} orders={ORDER_OPTIONS} state={state} shown={shownDirection} />
      <TabFilterSet
        tab="tests"
        boardKey={boardKey}
        fields={testsFields({ threads, subjects, plans })}
        filters={state.filters}
        patchFilters={state.patchFilters}
        placeholder={FIND_TEXT}
      />
      <Card>
        <ViewRow state={state} view={view} />
      </Card>
    </>
  );
};

export { TestsToolbar };
