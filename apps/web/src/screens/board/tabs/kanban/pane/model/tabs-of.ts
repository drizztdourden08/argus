/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../data/kanban';
import { PANE_FALLBACK_TAB, PANE_TABS } from '../pane.constants';
import { paneType } from './pane-type';

const tabsOf = (item: ItemRef): string[] => {
  const known = (paneType(item)?.tabs ?? []).filter((id) => Object.hasOwn(PANE_TABS, id));
  return known.length > 0 ? known : [PANE_FALLBACK_TAB];
};

export { tabsOf };
