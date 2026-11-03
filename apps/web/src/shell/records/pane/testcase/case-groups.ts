/* @layer renderer-app @kind logic */
import { itemTypeOf } from '../../../../data/registry';
import type { FieldGroupModel } from '../fields/fields.type';
import { fieldGroups } from '../model/field-groups';
import { fieldText } from '../model/field-text';
import { CASE_FACT_FIELDS, CASE_KIND } from '../model/pane-model.constants';

const caseGroups = (values: Readonly<Record<string, unknown>>): FieldGroupModel[] => {
  const type = itemTypeOf(CASE_KIND);
  if (!type) return [];
  return fieldGroups(type)
    .map((group) => ({ ...group, fields: group.fields.filter((name) => !CASE_FACT_FIELDS.includes(name)) }))
    .filter((group) => group.fields.some((name) => fieldText(name, values[name]) !== ''));
};

export { caseGroups };
