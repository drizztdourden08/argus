/* @layer renderer-app @kind logic */
import type { KanbanConfig } from '../../../../../data/types';
import { configUniqueId } from './config-unique-id';

const configAddRule = (config: KanbanConfig): KanbanConfig => {
  const rules = config.rules ?? [];
  const id = configUniqueId(rules.map((rule) => rule.id), 'rule');
  return { ...config, rules: [...rules, { id, kind: 'limit', params: {}, mode: 'warn' }] };
};

export { configAddRule };
