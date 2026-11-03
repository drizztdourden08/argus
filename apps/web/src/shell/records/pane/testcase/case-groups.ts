/* @layer renderer-app @kind logic */
import { itemTypeOf } from '../../../../data/registry';
import type { FieldGroupModel } from '../fields/fields.type';
import { fieldGroups } from '../model/field-groups';
import { CASE_FACT_FIELDS, CASE_KIND } from '../model/pane-model.constants';

const caseGroups = (): FieldGroupModel[] => {
  const type = itemTypeOf(CASE_KIND);
  if (!type) return [];
  return fieldGroups(type)
    .map((group) => ({ ...group, fields: group.fields.filter((name) => !CASE_FACT_FIELDS.includes(name)) }))
    .filter((group) => group.fields.length > 0);
};

export { caseGroups };
