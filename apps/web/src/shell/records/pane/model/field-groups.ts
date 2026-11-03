/* @layer renderer-app @kind logic */
import { ITEM_REGISTRY } from '../../../../data/registry';
import type { ItemType } from '../../../../data/registry';
import { TAB_FIELDS } from './pane-model.constants';
import type { FieldGroupModel } from '../fields/fields.type';

const shown = (id: string): boolean => !ITEM_REGISTRY.baseFields.includes(id) && !TAB_FIELDS.includes(id);

const fieldGroups = (type: ItemType): FieldGroupModel[] => {
  const groups = (type.groups ?? [])
    .map((group) => ({ id: group.id, label: group.label, fields: group.fields.filter(shown) }))
    .filter((group) => group.fields.length > 0);
  const grouped = new Set(groups.flatMap((group) => group.fields));
  const rest = type.fields.filter((id) => shown(id) && !grouped.has(id));
  if (rest.length === 0) return groups;
  return [...groups, { id: 'more', label: groups.length > 0 ? 'More' : 'Details', fields: rest }];
};

export { fieldGroups };
