/* @layer renderer-app @kind logic */
import type { KanbanColumnConfig, KanbanConfig } from '../../../../../data/types';

const columnsOf = (config: KanbanConfig | null | undefined): KanbanColumnConfig[] | null =>
  config && Array.isArray(config.columns) && config.columns.length > 0 ? config.columns : null;

export { columnsOf };
