/* @layer renderer-app @kind logic */
import { itemTypeOf } from './item-type-of';
import { ITEM_REGISTRY } from './registry.constants';

const typeHasField = (typeId: string, fieldId: string): boolean =>
  (itemTypeOf(typeId)?.fields.includes(fieldId) ?? false) || ITEM_REGISTRY.baseFields.includes(fieldId);

export { typeHasField };
