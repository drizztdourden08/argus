/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { fieldOf, itemTypeOf } from '../../../../../data/registry';
import type { ItemField } from '../../../../../data/registry';
import { CHIP_PREFIX } from './model.constants';
import { typeOf } from './type-of';
import type { CardChip } from './model.type';

const meaning = (field: ItemField | undefined, value: string): string => {
  const label = field?.labels?.[field.values?.indexOf(value) ?? -1];
  return label ? `${value} (${label})` : value;
};

const chipOf = (id: string, value: unknown): CardChip | null => {
  if (typeof value !== 'string' || value === '') return null;
  const name = `${id.charAt(0).toUpperCase()}${id.slice(1)}`;
  return { id, text: `${CHIP_PREFIX[id] ?? ''}${value}`, title: `${name}: ${meaning(fieldOf(id), value)}` };
};

const cardChips = (item: ItemRef): CardChip[] =>
  (itemTypeOf(typeOf(item))?.card.chips ?? []).flatMap((id) => chipOf(id, item.fields?.[id]) ?? []);

export { cardChips };
