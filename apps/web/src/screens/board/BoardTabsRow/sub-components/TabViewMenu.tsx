/* @layer renderer-app @kind component */
import { ANY_TAB } from '../BoardTabsRow.constants';
import type { TabViewMenuProps } from '../BoardTabsRow.type';
import { useTabTools } from '../behavior/useTabTools';
import { TabGroupSort } from './TabGroupSort';

const TabViewMenu = <G extends string, O extends string>({ groups, orders, state, shown }: TabViewMenuProps<G, O>) => {
  const { prefs, setGroup, setOrder, setDirection } = state;
  const direction = shown(prefs.order, prefs.direction);
  const pickSort = (order: O, next: typeof direction) => (order === prefs.order ? setDirection(next) : setOrder(order));
  useTabTools(ANY_TAB, [
    {
      id: 'group-sort',
      priority: 5,
      node: <TabGroupSort group={{ options: groups, value: prefs.group, onChange: setGroup }} sort={{ options: orders, order: prefs.order, direction, onChange: pickSort }} />,
    },
  ]);
  return null;
};

export { TabViewMenu };
