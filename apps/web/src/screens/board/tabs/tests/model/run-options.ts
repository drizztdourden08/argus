/* @layer renderer-app @kind logic */
import { itemCode } from '../../../../../shell';
import type { ItemDoc } from './model.type';

const runOptions = (items: readonly ItemDoc[]): { value: string; label: string }[] =>
  items
    .filter((item) => item.type === 'test-run' && !item.deletedAt)
    .map((item) => ({ value: item.id, label: `${itemCode(item)} ${item.title}` }))
    .sort((a, b) => b.value.localeCompare(a.value, undefined, { numeric: true }));

export { runOptions };
