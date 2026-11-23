/* @layer renderer-app @kind component */
import { TabFilterBar, TabGroupSort, ToolToggle, useTabTools } from '../../BoardTabsRow';
import { LayoutToggle } from '../../shared';
import { LAYOUT_CHOICES } from './builds.constants';
import type { BuildsToolsProps } from './builds-props.type';
import { BUILD_GROUPS, BUILD_ORDERS, BUILDS_TAB, buildFields, SEARCH_PLACEHOLDER } from './model';

const BuildsTools = (props: BuildsToolsProps) => {
  const { boardKey, tabView, data, layout, closed } = props;
  const { group, setGroup, order, direction, setSort, filters, patchFilters } = tabView;
  useTabTools(BUILDS_TAB, [
    {
      id: 'group-sort',
      priority: 5,
      node: <TabGroupSort group={{ options: BUILD_GROUPS, value: group, empty: 'none', onChange: setGroup }} sort={{ options: BUILD_ORDERS, order, direction, onChange: setSort }} />,
    },
    { id: 'layout', priority: 4, node: <LayoutToggle value={layout.value} options={LAYOUT_CHOICES} onChange={layout.onChange} label="Build view" /> },
    { id: 'show-closed', priority: 3, node: <ToolToggle label="Show closed" count={closed.count} pressed={closed.shown} onToggle={closed.onToggle} /> },
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
