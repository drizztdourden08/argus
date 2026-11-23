/* @layer renderer-app @kind logic */
import type { KanbanColumnConfig, KanbanConfig } from '../../../../../data/types';

const configSetColumn = (config: KanbanConfig, index: number, column: KanbanColumnConfig): KanbanConfig => ({
  ...config,
  columns: config.columns.map((entry, at) => (at === index ? column : entry)),
});

export { configSetColumn };
