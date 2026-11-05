/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { SortDirection } from '@drizztdourden08/tessera';
import { TabFilterBar, TabGroupSort, ToolToggle, useTabTools } from '../../../BoardTabsRow';
import { filterFields } from '../model/filter-fields';
import { GROUP_OPTIONS, ORDER_OPTIONS, shownDirection } from '../model';
import type { TestOrder } from '../model';
import { FIND_TEXT } from '../tests.constants';
import type { TestsToolbarProps } from '../tests-props.type';

const TestsToolbar = (props: TestsToolbarProps) => {
  const { boardKey, state, view, threads, subjects, plans, runs } = props;
  const { prefs } = state;
  const sortBy = (order: TestOrder, direction: SortDirection) => {
    if (order !== prefs.order) state.setOrder(order);
    state.setDirection(direction);
  };
  const counts = `${view.toRun} to run / ${view.total}${view.filtered ? ' · filtered' : ''}`;
  useTabTools('tests', [
    {
      id: 'group-sort',
      priority: 5,
      node: (
        <TabGroupSort
          group={{ options: GROUP_OPTIONS, value: prefs.group, onChange: state.setGroup }}
          sort={{ options: ORDER_OPTIONS, order: prefs.order, direction: shownDirection(prefs.order, prefs.direction), onChange: sortBy }}
        />
      ),
    },
    { id: 'show-passed', priority: 3, node: <ToolToggle label="Show passed" pressed={prefs.showPassed} onToggle={() => state.setShowPassed(!prefs.showPassed)} /> },
    { id: 'counts', priority: 1, node: <Text variant="caption" tone="muted" aria-live="polite">{counts}</Text> },
  ]);
  return (
    <TabFilterBar
      tab="tests"
      boardKey={boardKey}
      fields={filterFields({ threads, runs, subjects, plans })}
      filters={state.filters}
      patchFilters={state.patchFilters}
      placeholder={FIND_TEXT}
    />
  );
};

export { TestsToolbar };
