/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import { kindTabs } from './kind-tabs';
import { paneType } from './pane-type';

const tabsOf = (item: ItemRef): string[] => {
  const type = paneType(item);
  return type ? kindTabs(type) : [];
};

export { tabsOf };
