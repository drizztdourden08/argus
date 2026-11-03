/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../data/kanban';
import { PANE_TABS } from '../pane.constants';
import { paneType } from './pane-type';

const tabsOf = (item: ItemRef): string[] => (paneType(item)?.tabs ?? []).filter((id) => Object.hasOwn(PANE_TABS, id));

export { tabsOf };
