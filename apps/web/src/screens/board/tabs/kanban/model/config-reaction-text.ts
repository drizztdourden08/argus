/* @layer renderer-app @kind logic */
import type { KanbanRuleConfig } from '../../../../../data/types';
import { HOUR_MS, NEW_ALERT } from './config-editor.constants';
import { configKind } from './config-kind';

const configReactionText = (rule: KanbanRuleConfig): string => {
  if (!configKind(rule.kind).sweep) return `Mode: ${rule.mode ?? 'warn'}`;
  const alert = rule.alert ?? NEW_ALERT;
  return `Alerts ${alert.to} every ${alert.repeatMs / HOUR_MS} h${alert.wake ? ' and wakes the thread' : ''}`;
};

export { configReactionText };
