/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { KanbanConfig } from '../../src/data/types';
import {
  HOUR_MS, configAddColumn, configAddRule, configMoveColumn, configParseList, configRemoveColumn, configRemoveRule, configRuleOfKind, configSetColumn, configSetRule, configToggleList,
} from '../../src/screens/board/tabs/kanban/model';

const base: KanbanConfig = {
  columns: [
    { id: 'a', name: 'A', match: { lanes: ['todo'] }, rules: ['r1', 'r2'] },
    { id: 'b', name: 'B', match: { lanes: ['doing'] }, rules: ['r1'] },
  ],
  rules: [
    { id: 'r1', kind: 'limit', params: {}, mode: 'warn' },
    { id: 'r2', kind: 'dwell', params: { ms: HOUR_MS } },
  ],
};

describe('I-206 the config editor helpers', { tags: ['critical'] }, () => {
  it('adds a column and a rule with a free id', () => {
    const next = configAddRule(configAddColumn(configAddColumn(base)));
    expect(next.columns.map((column) => column.id)).toEqual(['a', 'b', 'column', 'column-2']);
    expect(next.rules?.map((rule) => rule.id)).toEqual(['r1', 'r2', 'rule']);
  });

  it('moves a column and stops at the ends', () => {
    expect(configMoveColumn(base, 0, 1).columns.map((column) => column.id)).toEqual(['b', 'a']);
    expect(configMoveColumn(base, 0, -1)).toBe(base);
    expect(configMoveColumn(base, 1, 1)).toBe(base);
  });

  it('removes a column, and a rule from every column that carried it', () => {
    expect(configRemoveColumn(base, 0).columns.map((column) => column.id)).toEqual(['b']);
    const next = configRemoveRule(base, 0);
    expect(next.rules?.map((rule) => rule.id)).toEqual(['r2']);
    expect(next.columns.map((column) => column.rules)).toEqual([['r2'], []]);
  });

  it('renames a rule in the columns that name it', () => {
    const next = configSetRule(base, 0, { id: 'cap', kind: 'limit', params: {}, mode: 'refuse' });
    expect(next.columns.map((column) => column.rules)).toEqual([['cap', 'r2'], ['cap']]);
    expect(configSetColumn(base, 1, { ...base.columns[1], name: 'Busy' }).columns[1].name).toBe('Busy');
  });

  it('gives a rule the defaults of its new kind', () => {
    const sweep = configRuleOfKind(base.rules?.[0] ?? { id: 'x', kind: 'limit' }, 'dwell');
    expect(sweep).toMatchObject({ id: 'r1', kind: 'dwell', params: { ms: 4 * HOUR_MS }, alert: { to: 'item-thread', wake: true } });
    expect(sweep.mode).toBeUndefined();
    const write = configRuleOfKind(sweep, 'require-field');
    expect(write).toMatchObject({ kind: 'require-field', params: {}, mode: 'warn' });
    expect(write.alert).toBeUndefined();
  });

  it('parses comma lists and toggles list entries', () => {
    expect(configParseList(' epic, task ,, ')).toEqual(['epic', 'task']);
    expect(configToggleList(['todo'], 'doing', true)).toEqual(['todo', 'doing']);
    expect(configToggleList(['todo', 'doing'], 'todo', false)).toEqual(['doing']);
  });
});
