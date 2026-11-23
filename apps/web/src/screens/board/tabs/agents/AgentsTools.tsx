/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { TabFilterBar, TabGroupSort, ToolToggle, useTabTools } from '../../BoardTabsRow';
import { COLLAPSE_ALL, EXPAND_ALL } from './AgentsTab.constants';
import { LayoutToggle } from '../../shared';
import type { AgentsToolsProps } from './AgentsTab.type';
import { agentFields, AGENT_GROUPS, AGENT_ORDERS, AGENTS_TAB, LAYOUT_CHOICES, SEARCH_PLACEHOLDER } from './model';

const AgentsTools = (props: AgentsToolsProps) => {
  const { boardKey, tabView, view, finished, layout, expand } = props;
  const { group, setGroup, order, direction, setSort, filters, patchFilters } = tabView;
  const label = expand.all ? COLLAPSE_ALL : EXPAND_ALL;
  const expandTool = {
    id: 'expand-all',
    priority: 2,
    node: (
      <IconButton size="sm" variant="ghost" label={label} title={label} aria-label={label} aria-pressed={expand.all} onClick={expand.onToggle}>
        <Icon name={expand.all ? 'chevrons-down-up' : 'chevrons-up-down'} />
      </IconButton>
    ),
  };
  useTabTools(AGENTS_TAB, [
    {
      id: 'group-sort',
      priority: 5,
      node: <TabGroupSort group={{ options: AGENT_GROUPS, value: group, empty: 'none', onChange: setGroup }} sort={{ options: AGENT_ORDERS, order, direction, onChange: setSort }} />,
    },
    { id: 'layout', priority: 4, node: <LayoutToggle value={layout.value} options={LAYOUT_CHOICES} onChange={layout.onChange} label="Agent view" /> },
    { id: 'show-finished', priority: 3, node: <ToolToggle label="Show finished" count={finished.count} pressed={finished.shown} onToggle={finished.onToggle} /> },
    ...(layout.value === 'list' ? [expandTool] : []),
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
