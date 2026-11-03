/* @layer renderer-app @kind logic */
import type { ItemDoc } from './model.type';

const planOptions = (items: readonly ItemDoc[]): { value: string; label: string }[] =>
  items
    .filter((item) => item.type === 'test-plan' && !item.deletedAt)
    .map((item) => ({ value: item.id, label: `${item.id} ${item.title}` }))
    .sort((a, b) => a.value.localeCompare(b.value, undefined, { numeric: true }));

export { planOptions };
