/* @layer renderer-app @kind logic */
import type { KanbanConfig } from '../../../../../data/types';

const configRemoveColumn = (config: KanbanConfig, index: number): KanbanConfig => ({
  ...config,
  columns: config.columns.filter((_, at) => at !== index),
});

export { configRemoveColumn };
