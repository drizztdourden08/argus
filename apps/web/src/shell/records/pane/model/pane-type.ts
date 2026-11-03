/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import { itemTypeOf } from '../../../../data/registry';
import type { ItemType } from '../../../../data/registry';
import { DEFAULT_TYPE, typeOf } from '../../../../screens/board/tabs/kanban/model';

const paneType = (item: ItemRef): ItemType | undefined => itemTypeOf(typeOf(item)) ?? itemTypeOf(DEFAULT_TYPE);

export { paneType };
