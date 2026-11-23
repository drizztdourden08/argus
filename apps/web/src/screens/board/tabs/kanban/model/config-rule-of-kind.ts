/* @layer renderer-app @kind logic */
import type { KanbanRuleConfig } from '../../../../../data/types';
import { HOUR_MS, NEW_ALERT } from './config-editor.constants';
import { configKind } from './config-kind';

const configRuleOfKind = (rule: KanbanRuleConfig, kind: string): KanbanRuleConfig => {
  const entry = configKind(kind);
  const params = entry.params.some((field) => field.key === 'ms') ? { ms: 4 * HOUR_MS } : {};
  return entry.sweep ? { id: rule.id, kind, params, alert: NEW_ALERT } : { id: rule.id, kind, params, mode: 'warn' };
};

export { configRuleOfKind };
