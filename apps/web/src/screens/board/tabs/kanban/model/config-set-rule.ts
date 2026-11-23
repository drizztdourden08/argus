/* @layer renderer-app @kind logic */
import type { KanbanConfig, KanbanRuleConfig } from '../../../../../data/types';

const configSetRule = (config: KanbanConfig, index: number, rule: KanbanRuleConfig): KanbanConfig => {
  const rules = config.rules ?? [];
  const was = rules[index]?.id;
  const columns = was === rule.id
    ? config.columns
    : config.columns.map((column) => ({ ...column, rules: (column.rules ?? []).map((id) => (id === was ? rule.id : id)) }));
  return { columns, rules: rules.map((entry, at) => (at === index ? rule : entry)) };
};

export { configSetRule };
