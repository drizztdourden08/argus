/* @layer renderer-app @kind logic */
import { itemCode } from '../../../../../shell';
import type { ItemDoc } from './model.type';

const ITEM_TITLE_CHARS = 40;

const itemChoices = (items: readonly ItemDoc[]): { value: string; label: string }[] =>
  items.filter((item) => !item.deletedAt).map((item) => ({ value: item.id, label: `${itemCode(item)} ${item.title.slice(0, ITEM_TITLE_CHARS)}` }));

export { itemChoices };
