/* @layer renderer-app @kind component */
import { TabFilterBar, TabGroupSort, ToolToggle, useTabTools } from '../../BoardTabsRow';
import type { AgentsToolsProps } from './AgentsTab.type';
import { agentFields, AGENT_GROUPS, AGENT_ORDERS, AGENTS_TAB, SEARCH_PLACEHOLDER } from './model';

const AgentsTools = (props: AgentsToolsProps) => {
  const { boardKey, tabView, view, finished } = props;
  const { group, setGroup, order, direction, setSort, filters, patchFilters } = tabView;
  useTabTools(AGENTS_TAB, [
    {
      id: 'group-sort',
      priority: 5,
      node: <TabGroupSort group={{ options: AGENT_GROUPS, value: group, empty: 'none', onChange: setGroup }} sort={{ options: AGENT_ORDERS, order, direction, onChange: setSort }} />,
    },
    { id: 'show-finished', priority: 3, node: <ToolToggle label="Show finished" count={finished.count} pressed={finished.shown} onToggle={finished.onToggle} /> },
  ]);
  return (
    <TabFilterBar
      tab={AGENTS_TAB}
      boardKey={boardKey}
      fields={agentFields(view.threads, view.items)}
      filters={filters}
      patchFilters={patchFilters}
      placeholder={SEARCH_PLACEHOLDER}
    />
  );
};

export { AgentsTools };
