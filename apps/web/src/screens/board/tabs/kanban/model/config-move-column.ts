/* @layer renderer-app @kind logic */
import type { KanbanConfig } from '../../../../../data/types';

const configMoveColumn = (config: KanbanConfig, index: number, step: -1 | 1): KanbanConfig => {
  const to = index + step;
  if (to < 0 || to >= config.columns.length) return config;
  const columns = [...config.columns];
  const [moved] = columns.splice(index, 1);
  if (!moved) return config;
  columns.splice(to, 0, moved);
  return { ...config, columns };
};

export { configMoveColumn };
