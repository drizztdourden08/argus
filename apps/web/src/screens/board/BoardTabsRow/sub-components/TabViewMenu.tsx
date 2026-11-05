/* @layer renderer-app @kind component */
import { ViewControls } from '@drizztdourden08/tessera';
import type { SortState } from '@drizztdourden08/tessera';
import type { TabViewMenuProps } from '../BoardTabsRow.type';
import { TabTools } from './TabTools';

const TabViewMenu = <G extends string, O extends string>({ groups, orders, state, shown }: TabViewMenuProps<G, O>) => {
  const { prefs, setGroup, setOrder, setDirection } = state;
  const sort: SortState = { key: prefs.order, direction: shown(prefs.order, prefs.direction) };
  const pickGroup = (id: string | null) => {
    const next = groups.find((option) => option.id === id);
    if (next) setGroup(next.id);
  };
  const pickOrder = (change: SortState) => {
    const next = orders.find((option) => option.id === change.key);
    if (!next) return;
    if (next.id === prefs.order) setDirection(change.direction);
    else setOrder(next.id);
  };
  return (
    <TabTools>
      <ViewControls
        group={{ options: groups, value: prefs.group, onChange: pickGroup }}
        sort={{ options: orders, value: sort, onChange: pickOrder }}
      />
    </TabTools>
  );
};

export { TabViewMenu };
