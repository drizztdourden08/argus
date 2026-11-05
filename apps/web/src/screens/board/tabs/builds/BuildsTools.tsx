/* @layer renderer-app @kind component */
import { TabFilterBar, TabGroupSort, useTabTools } from '../../BoardTabsRow';
import type { BuildsToolsProps } from './builds-props.type';
import { BUILD_GROUPS, BUILD_ORDERS, BUILDS_TAB, buildFields, SEARCH_PLACEHOLDER } from './model';

const BuildsTools = (props: BuildsToolsProps) => {
  const { boardKey, tabView, data } = props;
  const { group, setGroup, order, direction, setSort, filters, patchFilters } = tabView;
  useTabTools(BUILDS_TAB, [
    {
      id: 'group-sort',
      priority: 5,
      node: <TabGroupSort group={{ options: BUILD_GROUPS, value: group, empty: 'none', onChange: setGroup }} sort={{ options: BUILD_ORDERS, order, direction, onChange: setSort }} />,
    },
  ]);
  return (
    <TabFilterBar
      tab={BUILDS_TAB}
      boardKey={boardKey}
      fields={buildFields(data.views, data.items)}
      filters={filters}
      patchFilters={patchFilters}
      placeholder={SEARCH_PLACEHOLDER}
    />
  );
};

export { BuildsTools };
