/* @layer renderer-app @kind logic */
import type { KanbanRuleConfig } from '../../../../../data/types';
import type { RuleFact } from '../../../../../shell';
import { HOUR_MS } from './config-editor.constants';
import type { ParamField } from './config-editor.type';
import { configKind } from './config-kind';

const paramText = (field: ParamField, value: unknown): string => {
  if (field.type === 'hours') return typeof value === 'number' ? `${value / HOUR_MS} h` : 'none';
  if (field.type === 'list') return (Array.isArray(value) ? value.map(String).join(', ') : '') || 'any';
  return typeof value === 'string' && value ? value : 'none';
};

const configRuleFacts = (rule: KanbanRuleConfig): RuleFact[] =>
  configKind(rule.kind).params.map((field) => ({ label: field.label, value: paramText(field, rule.params?.[field.key]) }));

export { configRuleFacts };
