/* @layer renderer-app @kind component */
import { ViewControls } from '@drizztdourden08/tessera';
import type { SortState } from '@drizztdourden08/tessera';
import { TabTools } from '../../../BoardTabsRow';
import type { ViewMenuProps } from '../kanban-props.type';
import { GROUP_OPTIONS, ORDER_OPTIONS } from '../model';

const SORT_DIRECTION = { rank: 'asc', updated: 'desc' } as const;

const ViewMenu = ({ state }: ViewMenuProps) => {
  const { prefs, setGroup, setOrder } = state;
  const sort: SortState = { key: prefs.order, direction: SORT_DIRECTION[prefs.order] };
  const pickGroup = (id: string | null) => {
    const next = GROUP_OPTIONS.find((option) => option.id === id);
    if (next) setGroup(next.id);
  };
  const pickOrder = (change: SortState) => {
    const next = ORDER_OPTIONS.find((option) => option.id === change.key);
    if (next) setOrder(next.id);
  };
  return (
    <TabTools>
      <ViewControls
        group={{ options: GROUP_OPTIONS, value: prefs.group, onChange: pickGroup }}
        sort={{ options: ORDER_OPTIONS, value: sort, onChange: pickOrder }}
      />
    </TabTools>
  );
};

export { ViewMenu };
