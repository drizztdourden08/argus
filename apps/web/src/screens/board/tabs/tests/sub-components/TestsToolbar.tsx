/* @layer renderer-app @kind component */
import type { SortDirection } from '@drizztdourden08/tessera';
import { TabFilterBar, TabGroupSort, ToolToggle, useTabTools } from '../../../BoardTabsRow';
import { LayoutToggle } from '../../../shared';
import { filterFields } from '../model/filter-fields';
import { GROUP_OPTIONS, ORDER_OPTIONS, shownDirection, TESTS_TAB } from '../model';
import type { TestOrder } from '../model';
import { FIND_TEXT, LAYOUT_CHOICES } from '../tests.constants';
import type { TestsToolbarProps } from '../tests-props.type';

const TestsToolbar = (props: TestsToolbarProps) => {
  const { boardKey, state, threads, subjects, plans, runs, closedCount } = props;
  const { prefs } = state;
  const sortBy = (order: TestOrder, direction: SortDirection) => {
    if (order !== prefs.order) {
      state.setOrder(order);
      return;
    }
    state.setDirection(direction);
  };
  useTabTools(TESTS_TAB, [
    {
      id: 'group-sort',
      priority: 5,
      node: (
        <TabGroupSort
          group={{ options: GROUP_OPTIONS, value: prefs.group, empty: 'run', onChange: state.setGroup }}
          sort={{ options: ORDER_OPTIONS, order: prefs.order, direction: shownDirection(prefs.order, prefs.direction), onChange: sortBy }}
        />
      ),
    },
    { id: 'layout', priority: 4, node: <LayoutToggle value={state.layout} options={LAYOUT_CHOICES} onChange={state.setLayout} label="Case view" /> },
    { id: 'show-passed', priority: 3, node: <ToolToggle label="Show passed" pressed={prefs.showPassed} onToggle={() => state.setShowPassed(!prefs.showPassed)} /> },
    { id: 'show-closed', priority: 2, node: <ToolToggle label="Show closed" count={closedCount} pressed={prefs.showClosed} onToggle={() => state.setShowClosed(!prefs.showClosed)} /> },
  ]);
  return (
    <TabFilterBar
      tab={TESTS_TAB}
      boardKey={boardKey}
      fields={filterFields({ threads, runs, subjects, plans })}
      filters={state.filters}
      patchFilters={state.patchFilters}
      placeholder={FIND_TEXT}
    />
  );
};

export { TestsToolbar };
