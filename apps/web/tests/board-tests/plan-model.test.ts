/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import {
  buildView, caseAssignee, caseChips, DEFAULT_FILTERS, groupByPlan, planCode, planTone, planViews, shownIds,
} from '../../src/screens/board/tabs/tests/model';
import type { ItemDoc, TestDoc, TestPrefs, TestsView } from '../../src/screens/board/tabs/tests/model';
import { doneDoc, itemDoc, testDoc } from './fixtures';

const items = (...docs: object[]): ItemDoc[] => docs as ItemDoc[];
const tests = (...docs: object[]): TestDoc[] => docs as TestDoc[];

const planDoc = (id: string, cases: string[], extra: object = {}) =>
  itemDoc(id, { title: `Plan ${id}`, subject: null, type: 'test-plan', stage: 'ready', fields: { cases }, ...extra });

const run = (results: object[], endedAt: string | null = null) => ({ n: 1, bundle: null, by: 'human', startedAt: '2026-10-26T09:00:00.000Z', endedAt, results });

const prefs = (group: TestPrefs['group']): TestPrefs => ({ group, order: 'rank', showPassed: true, showClosed: false });

const viewOf = (group: TestPrefs['group'], docs: TestDoc[], plans: ItemDoc[]): TestsView =>
  buildView({ tests: docs, builds: [], items: plans, launchers: [], defs: [], handed: null, filters: DEFAULT_FILTERS, prefs: prefs(group), conflicted: new Set() });

describe('P5 the plans of the Tests tab', { tags: ['critical'] }, () => {
  it('lists the live test plans by id, with every case to run when no run was started', () => {
    const plans = planViews(items(planDoc('I-10', ['T-1']), planDoc('I-9', ['T-1', 'T-2']), planDoc('I-7', [], { deletedAt: 'x' }), itemDoc('I-1')), '');
    expect(plans.map((plan) => plan.item.id)).toEqual(['I-9', 'I-10']);
    expect(plans[0]?.counts).toEqual({ run: 2, pass: 0, fail: 0, partial: 0, blocked: 0 });
    expect(plans[0]?.live).toBe(false);
  });

  it('takes the progress from the run in progress and marks the plan live', () => {
    const live = planDoc('I-5', ['T-1', 'T-2', 'T-3'], { fields: { cases: ['T-1', 'T-2', 'T-3'], runs: [run([{ test: 'T-1', status: 'pass', note: '', at: 'x' }, { test: 'T-2', status: 'fail', note: 'no', at: 'x' }])] } });
    const [plan] = planViews(items(live), '');
    expect(plan?.counts).toEqual({ run: 1, pass: 1, fail: 1, partial: 0, blocked: 0 });
    expect(plan?.live).toBe(true);
  });

  it('keeps the last finished run for a plan with no run in progress', () => {
    const ended = planDoc('I-5', ['T-1'], { fields: { cases: ['T-1'], runs: [run([{ test: 'T-1', status: 'pass', note: '', at: 'x' }], '2026-10-26T10:00:00.000Z')] } });
    const [plan] = planViews(items(ended), '');
    expect(plan?.counts.pass).toBe(1);
    expect(plan?.live).toBe(false);
  });

  it('narrows to the plan of the plan filter, and marks a passed or abandoned plan closed', () => {
    const docs = items(planDoc('I-5', []), planDoc('I-6', [], { stage: 'passed' }), planDoc('I-7', [], { stage: 'abandoned' }));
    expect(planViews(docs, 'I-6').map((plan) => plan.item.id)).toEqual(['I-6']);
    expect(planViews(docs, '').map((plan) => plan.closed)).toEqual([false, true, true]);
  });

  it('wears a tone from the worst outcome: failed, then partial or blocked, then all passed', () => {
    expect(planTone({ run: 1, pass: 1, fail: 1, partial: 0, blocked: 0 })).toBe('danger');
    expect(planTone({ run: 0, pass: 1, fail: 0, partial: 1, blocked: 0 })).toBe('warning');
    expect(planTone({ run: 0, pass: 2, fail: 0, partial: 0, blocked: 0 })).toBe('success');
    expect(planTone({ run: 2, pass: 0, fail: 0, partial: 0, blocked: 0 })).toBeUndefined();
  });

  it('shows a plan by its code of the registry', () => {
    expect(planCode('I-5')).toBe('TP-5');
  });
});

describe('P5 the cases grouped by plan', { tags: ['critical'] }, () => {
  const plans = items(planDoc('I-5', ['T-1', 'T-2']), planDoc('I-6', ['T-2']), planDoc('I-8', ['T-4'], { stage: 'passed' }));
  const docs = tests(testDoc('T-1'), testDoc('T-2'), testDoc('T-3'), testDoc('T-4'));
  const grouped = groupByPlan(docs, planViews(plans, ''));

  it('puts a case under every plan that holds it', () => {
    expect(grouped.groups.map((group) => [group.key, group.rows.map((row) => row.id)])).toEqual([['I-5', ['T-1', 'T-2']], ['I-6', ['T-2']]]);
  });

  it('keeps the cases of no plan apart, and the cases of a closed plan in the closed groups', () => {
    expect(grouped.none.map((row) => row.id)).toEqual(['T-3']);
    expect(grouped.closed.map((group) => [group.key, group.rows.map((row) => row.id)])).toEqual([['I-8', ['T-4']]]);
  });

  it('is empty outside the plan grouping, and the view still lists the plans', () => {
    const view = viewOf('build', docs, plans);
    expect(view.byPlan.groups.length).toBe(0);
    expect(view.byPlan.none.length).toBe(0);
    expect(view.plans.length).toBe(3);
  });

  it('is filled by the view in the plan grouping', () => {
    const view = viewOf('plan', docs, plans);
    expect(view.byPlan.groups.length).toBe(2);
    expect(view.cards.length).toBe(0);
  });

  it('numbers the records of the pane for Previous and Next: the plans first, then each case once', () => {
    expect(shownIds(viewOf('plan', docs, plans), 'plan')).toEqual(['TP-5', 'TP-6', 'TP-8', 'T-1', 'T-2', 'T-4', 'T-3']);
  });
});

describe('P5 the head of a case row', { tags: ['critical'] }, () => {
  it('is assigned to the owner when the test is manual, else to its thread', () => {
    expect(caseAssignee(testDoc('T-1', { mode: 'manual', thread: 'lead' }) as TestDoc)).toBe('owner');
    expect(caseAssignee(testDoc('T-2', { mode: 'auto', thread: 'lead' }) as TestDoc)).toBe('lead');
    expect(caseAssignee(testDoc('T-3', { mode: 'auto', thread: null }) as TestDoc)).toBe('');
  });

  it('holds conflict, item, thread, auto and archived chips, leaving out the fact of the group', () => {
    const doc = doneDoc('T-1', 'pass', 'ok', { mode: 'auto', archivedAt: 'then' }) as TestDoc;
    expect(caseChips(doc, 'plan', new Set(['T-1'])).map((chip) => chip.id)).toEqual(['conflict', 'item', 'thread', 'auto', 'archived']);
    expect(caseChips(doc, 'item', new Set()).map((chip) => chip.id)).toEqual(['thread', 'auto', 'archived']);
    expect(caseChips(doc, 'thread', new Set()).map((chip) => chip.id)).toEqual(['item', 'auto', 'archived']);
  });
});
