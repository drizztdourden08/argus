/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { formatFilters } from '../../src/screens/board/shared';
import {
  changedFields, conflictedTests, DEFAULT_FILTERS, editProblem, filtersActive, latestWakes, parseStored, sortToRun, subjectsOf, testNotices, toRunCount, wakeChip,
} from '../../src/screens/board/tabs/tests/model';
import type { TestDoc, TestFilters, WakeDoc } from '../../src/screens/board/tabs/tests/model';
import { conflictEntry, doneDoc, itemDoc, testDoc, wakeDoc } from './fixtures';

const tests = (...docs: object[]): TestDoc[] => docs as TestDoc[];
const wakes = (...docs: object[]): WakeDoc[] => docs as WakeDoc[];

describe('K1 the saved view', { tags: ['critical'] }, () => {
  it('reads the Node keys and plan: q, threads, status, subject, plan, needs, conf, archived and sort', () => {
    const filters = JSON.stringify({ q: 'x', threads: ['a'], status: 'fail', subject: 's', plan: 'I-5', needs: true, conf: true, archived: true, sort: 'updated' });
    const view = parseStored({ filters, group: 'item', passed: '0', closed: '1' });
    expect(view.filters).toEqual({ query: 'x', threads: ['a'], status: 'fail', subject: 's', plan: 'I-5', needs: true, conflict: true, archived: true });
    expect(view.prefs).toEqual({ group: 'item', order: 'updated', showPassed: false, showClosed: true });
  });

  it('falls back to the defaults for nothing, damage or an unknown group', () => {
    expect(parseStored({ filters: '{oops', group: 'zzz', passed: null, closed: null })).toEqual({
      filters: DEFAULT_FILTERS,
      prefs: { group: 'run', order: 'rank', showPassed: true, showClosed: false },
    });
  });

  it('writes the filters back without dropping the keys it does not own', () => {
    const next: TestFilters = { ...DEFAULT_FILTERS, query: 'q', conflict: true };
    expect(JSON.parse(formatFilters('{"trash":true}', next, 'updated'))).toMatchObject({ trash: true, q: 'q', conf: true, sort: 'updated' });
  });

  it('counts a filter as on for everything but the archived switch', () => {
    expect(filtersActive(DEFAULT_FILTERS)).toBe(false);
    expect(filtersActive({ ...DEFAULT_FILTERS, archived: true })).toBe(false);
    expect(filtersActive({ ...DEFAULT_FILTERS, query: 'x' })).toBe(true);
  });
});

describe('K1 the order of the tests to run', { tags: ['critical'] }, () => {
  const docs = tests(testDoc('T-2', { rank: 1, createdAt: 'b', updatedAt: '2' }), testDoc('T-1', { rank: 1, createdAt: 'a', updatedAt: '1' }), testDoc('T-3', { rank: 0, updatedAt: '3' }));

  it('is the rank, then the creation time, then the id; or the newest update first', () => {
    expect(sortToRun(docs).map((test) => test.id)).toEqual(['T-3', 'T-1', 'T-2']);
    expect(sortToRun(docs, 'updated').map((test) => test.id)).toEqual(['T-3', 'T-2', 'T-1']);
  });
});

describe('K1 the subjects', { tags: ['critical'] }, () => {
  it('collects the subjects of the items and the tests, sorted and without blanks', () => {
    const items = [itemDoc('I-1', { subject: 'work' }), itemDoc('I-2', { subject: null })] as never[];
    expect(subjectsOf(items, tests(testDoc('T-1', { subject: 'play' }), testDoc('T-2', { subject: 'work' })))).toEqual(['play', 'work']);
  });

  it('counts the tests to run, archived ones left out', () => {
    const all = tests(testDoc('T-1'), testDoc('T-2'), doneDoc('T-3', 'pass', 'ok'), testDoc('T-4', { archivedAt: 'then' }));
    expect(toRunCount(all)).toBe(2);
  });
});

describe('K1 the edit of a test', { tags: ['critical'] }, () => {
  const test = testDoc('T-1', { covers: null }) as unknown as TestDoc;

  it('sends only what changed, trimmed', () => {
    expect(changedFields(test, { title: ' New ', covers: '', mode: 'manual' })).toEqual({ title: 'New' });
  });

  it('refuses an empty title and a text over its limit', () => {
    expect(editProblem({ title: ' ', covers: '', mode: 'auto' })).toBe('The title cannot be empty.');
    expect(editProblem({ title: 'ok', covers: 'x'.repeat(81), mode: 'auto' })).toMatch(/at most/);
    expect(editProblem({ title: 'ok', covers: '', mode: 'auto' })).toBeNull();
  });
});

describe('K1 the changes the board did not take', { tags: ['critical'] }, () => {
  const entries = (...docs: object[]) => docs as never[];

  it('keeps the conflicts and refusals of tests, one per sub-op of a bulk, and marks the conflicted tests', () => {
    const bulk = { id: 'O-5', kind: 'bulk', author: 'human', createdAt: 'a', ops: [{ kind: 'move', target: { c: 'tests', id: 'T-7' } }, { kind: 'move', target: { c: 'tests', id: 'T-8' } }], results: [{ status: 'conflict', by: 'lead' }, { status: 'applied' }] };
    const notices = testNotices(entries(conflictEntry('O-1', 'T-1'), conflictEntry('O-2', 'T-2', { status: 'refused' }), bulk), new Set(['O-2']));
    expect(notices.map((notice) => notice.id)).toEqual(['O-1', 'O-5#0']);
    expect([...conflictedTests(notices)]).toEqual(['T-1', 'T-7']);
  });
});

describe('K1 the wake chip', { tags: ['critical'] }, () => {
  const NOW = Date.parse('2026-10-26T10:00:00.000Z');
  const done = doneDoc('T-1', 'fail', 'x', { result: { status: 'fail', text: 'x', by: 'human', at: '2026-10-26T09:59:50.000Z' } }) as unknown as TestDoc;
  const chip = (list: WakeDoc[], at = NOW) => wakeChip({ test: done, wakes: list, latest: latestWakes(list), now: at });

  it('goes from waking to not listening once the coalescing window has passed', () => {
    const queued = wakes(wakeDoc('W-1', 'T-1', { status: 'queued', listeners: 0, at: '2026-10-26T09:59:55.000Z' }));
    expect(chip(queued)?.label).toBe('lead waking...');
    expect(chip(queued, NOW + 30000)?.label).toBe('lead not listening');
  });

  it('keeps the newest wake of a test', () => {
    const list = wakes(wakeDoc('W-1', 'T-1', { listeners: 0, at: '2026-10-26T09:59:51.000Z' }), wakeDoc('W-2', 'T-1', { at: '2026-10-26T09:59:58.000Z' }));
    expect(chip(list)?.label).toBe('lead notified');
  });

  it('has nothing for a coalesced wake whose target is gone', () => {
    expect(chip(wakes(wakeDoc('W-1', 'T-1', { status: 'coalesced', into: 'W-0', at: '2026-10-26T09:59:55.000Z' })))).toBeNull();
  });
});
