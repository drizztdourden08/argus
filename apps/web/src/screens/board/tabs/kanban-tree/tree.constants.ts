/* @layer renderer-app @kind constants */
import type { KanbanLayout, TreeCount } from './tree.type';

const LAYOUT_KEY = 'coord.layout';

const LAYOUT_OPTIONS: { value: KanbanLayout; label: string }[] = [
  { value: 'columns', label: 'Columns' },
  { value: 'tree', label: 'Tree' },
];

const CASE_TYPE = 'test-case';

const TREE_LABEL = 'Item tree';

const TREE_EMPTY = 'No item to show in the tree.';

const NO_COUNT: TreeCount = { done: 0, total: 0 };

const ITEM_DONE = 'done';

const CASE_DONE = 'pass';

const DROPPED = 'dropped';

const MANUAL_MODE = 'manual';

export { CASE_DONE, CASE_TYPE, DROPPED, ITEM_DONE, LAYOUT_KEY, LAYOUT_OPTIONS, MANUAL_MODE, NO_COUNT, TREE_EMPTY, TREE_LABEL };
