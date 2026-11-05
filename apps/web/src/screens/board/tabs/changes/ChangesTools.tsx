/* @layer renderer-app @kind component */
import { TabFilterBar, TabGroupSort, useTabTools } from '../../BoardTabsRow';
import { CHANGES_GROUPS, CHANGES_ORDERS, CHANGES_TAB } from './changes.constants';
import type { ChangesToolsProps } from './changes.type';
import { changesFields } from './model';

const ChangesTools = (props: ChangesToolsProps) => {
  const { boardKey, view, prs, items } = props;
  const sort = { options: CHANGES_ORDERS, order: view.order, direction: view.direction, onChange: view.setSort };
  const group = { options: CHANGES_GROUPS, value: view.group, onChange: view.setGroup };
  useTabTools(CHANGES_TAB, [{ id: 'changes-view', priority: 5, node: <TabGroupSort group={group} sort={sort} /> }]);
  return (
    <TabFilterBar
      tab={CHANGES_TAB}
      boardKey={boardKey}
      fields={changesFields(prs, items)}
      filters={view.filters}
      patchFilters={view.patchFilters}
      placeholder="Search title or branch"
    />
  );
};

export { ChangesTools };
