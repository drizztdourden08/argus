/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import { boardTabIds } from '../board/behavior/board-tab-ids';
import { BOARD_KIND } from '../board/board.constants';
import { kindTabs } from './kind-tabs';
import { paneType } from './pane-type';

const tabsOf = (item: ItemRef): string[] => {
  const type = paneType(item);
  if (!type) return [];
  return type.id === BOARD_KIND ? boardTabIds() : kindTabs(type);
};

export { tabsOf };
