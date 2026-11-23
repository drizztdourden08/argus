/* @layer renderer-app @kind logic */
import type { ColumnLimit } from './model.type';

const limitTone = (count: number, limit: ColumnLimit): 'info' | 'warning' | 'danger' => {
  if (count <= limit.max) return 'info';
  return limit.mode === 'warn' ? 'warning' : 'danger';
};

export { limitTone };
