/* @layer renderer-app @kind logic */
import type { KanbanConfig } from '../../../../../data/types';

const configRemoveRule = (config: KanbanConfig, index: number): KanbanConfig => {
  const rules = config.rules ?? [];
  const gone = rules[index]?.id;
  return {
    columns: config.columns.map((column) => ({ ...column, rules: (column.rules ?? []).filter((id) => id !== gone) })),
    rules: rules.filter((_, at) => at !== index),
  };
};

export { configRemoveRule };
