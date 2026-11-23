/* @layer renderer-app @kind constants */
import type { KanbanConfig, KanbanRuleAlert, KanbanRuleConfig } from '../../../../../data/types';
import type { RuleKind } from './config-editor.type';

const HOUR_MS = 3_600_000;

const LANES = ['todo', 'doing', 'review', 'done', 'dropped'];

const LIMIT_MODES = ['warn', 'refuse', 'alert'];

const WRITE_MODES = ['warn', 'refuse'];

const FIRST_KIND: RuleKind = { kind: 'require-parent', sweep: false, params: [{ key: 'types', label: 'Parent types', type: 'list' }] };

const RULE_KINDS: readonly RuleKind[] = [
  FIRST_KIND,
  { kind: 'require-assignee', sweep: false, params: [{ key: 'kinds', label: 'Assignee kinds', type: 'list' }] },
  { kind: 'limit', sweep: false, params: [] },
  { kind: 'require-field', sweep: false, params: [{ key: 'field', label: 'Field', type: 'text' }] },
  { kind: 'dwell', sweep: true, params: [{ key: 'ms', label: 'Hours in the lane', type: 'hours' }] },
  { kind: 'assignee-gone', sweep: true, params: [] },
  { kind: 'over-limit', sweep: true, params: [] },
  { kind: 'unassigned', sweep: true, params: [{ key: 'ms', label: 'Hours without an assignee', type: 'hours' }] },
];

const NEW_ALERT: KanbanRuleAlert = { to: 'item-thread', wake: true, repeatMs: 12 * HOUR_MS };

const ALERT_TARGET_HINT = 'item-thread, assignee, lead, owner or a thread id';

const stale = (id: string, hours: number): KanbanRuleConfig => ({ id, kind: 'dwell', params: { ms: hours * HOUR_MS }, alert: NEW_ALERT });

const DEFAULT_CONFIG: KanbanConfig = {
  columns: [
    { id: 'todo', name: 'To do', match: { lanes: ['todo'], stages: [] }, rules: ['stale-todo'] },
    { id: 'doing', name: 'Doing', match: { lanes: ['doing'], stages: [] }, rules: ['stale-doing'] },
    { id: 'review', name: 'Review', match: { lanes: ['review'], stages: [] }, rules: ['stale-review'] },
    { id: 'done', name: 'Done', match: { lanes: ['done', 'dropped'], stages: [] }, rules: [] },
  ],
  rules: [stale('stale-todo', 24), stale('stale-doing', 4), stale('stale-review', 6)],
};

export { ALERT_TARGET_HINT, DEFAULT_CONFIG, FIRST_KIND, HOUR_MS, LANES, LIMIT_MODES, NEW_ALERT, RULE_KINDS, WRITE_MODES };
