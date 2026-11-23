/* @layer renderer-app @kind logic */
import type { KanbanConfig } from '../../../../../data/types';

const configMoveColumn = (config: KanbanConfig, index: number, step: -1 | 1): KanbanConfig => {
  const to = index + step;
  if (to < 0 || to >= config.columns.length) return config;
  const columns = [...config.columns];
  [columns[index], columns[to]] = [columns[to], columns[index]];
  return { ...config, columns };
};

export { configMoveColumn };
