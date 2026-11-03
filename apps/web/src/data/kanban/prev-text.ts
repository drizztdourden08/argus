/* @layer renderer-app @kind logic */
import type { EditField, ItemRef } from './kanban.type';
import { FIELDS_PREFIX } from './prev-text.constants';

const storedValue = (item: ItemRef, field: EditField): unknown =>
  field.startsWith(FIELDS_PREFIX) ? item.fields?.[field.slice(FIELDS_PREFIX.length)] : item[field as keyof ItemRef];

const prevText = (item: ItemRef, field: EditField): string => {
  const raw = storedValue(item, field);
  if (typeof raw === 'string') return raw;
  return Array.isArray(raw) ? raw.join('\n') : '';
};

export { prevText };
