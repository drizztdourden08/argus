/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { ItemRef } from '../../src/data/kanban';
import { planRunOp } from '../../src/data/kanban';
import { caseSteps } from '../../src/screens/board/tabs/kanban/pane/run/model/case-steps';
import { firstOpen } from '../../src/screens/board/tabs/kanban/pane/run/model/first-open';
import { flaggedResults } from '../../src/screens/board/tabs/kanban/pane/run/model/flagged-results';
import { keyStatus } from '../../src/screens/board/tabs/kanban/pane/run/model/key-status';
import { lastResult } from '../../src/screens/board/tabs/kanban/pane/run/model/last-result';
import { noteProblem } from '../../src/screens/board/tabs/kanban/pane/run/model/note-problem';
import { runnerReducer } from '../../src/screens/board/tabs/kanban/pane/run/model/runner-reducer';
import { runsOf } from '../../src/screens/board/tabs/kanban/pane/run/model/runs-of';
import { shownRun } from '../../src/screens/board/tabs/kanban/pane/run/model/shown-run';
import { summarize } from '../../src/screens/board/tabs/kanban/pane/run/model/summarize';
import type { PlanRun, RunnerState } from '../../src/screens/board/tabs/kanban/pane/run/run.type';
import { mockPosts, onlyPost } from '../helpers/post-mock';

const IDS = ['T-1', 'T-2', 'T-3', 'T-4'];

const plan = (runs: unknown): ItemRef => ({ id: 'I-1', title: 'Plan', thread: 'worker', status: 'todo', fields: { cases: IDS, runs } });

const rawRun = (n: number, results: object[], extra: object = {}) => ({ n, bundle: 'b1', by: 'human', startedAt: 'a', endedAt: null, results, ...extra });

const result = (test: string, status: string, note = '') => ({ test, status, note, at: 'b' });

const runOf = (results: object[], extra: object = {}): PlanRun => {
  const [run] = runsOf(plan([rawRun(1, results, extra)]));
  if (!run) throw new Error('no run');
  return run;
};

describe('I-70 P7 reading the runs of a plan', { tags: ['critical'] }, () => {
  it('reads fields.runs and drops what is damaged', () => {
    const runs = runsOf(plan([rawRun(1, [result('T-1', 'pass'), { test: 'T-2', status: 'maybe' }, 7]), 'junk', { n: 'x' }]));
    expect(runs).toHaveLength(1);
    expect(runs[0]).toMatchObject({ n: 1, bundle: 'b1', endedAt: null, summary: null });
    expect(runs[0]?.results).toEqual([result('T-1', 'pass')]);
    expect(runsOf({ ...plan(null), fields: {} })).toEqual([]);
  });

  it('shows the open run first, else the latest one', () => {
    const open = rawRun(2, []);
    const done = (n: number) => rawRun(n, [], { endedAt: 'z' });
    expect(shownRun(runsOf(plan([done(1), open, done(3)])))?.n).toBe(2);
    expect(shownRun(runsOf(plan([done(3), done(1)])))?.n).toBe(3);
    expect(shownRun([])).toBeNull();
  });

  it('keeps the latest result of a case', () => {
    const run = runOf([result('T-1', 'fail', 'bad'), result('T-1', 'pass')]);
    expect(lastResult(run, 'T-1')?.status).toBe('pass');
    expect(lastResult(run, 'T-2')).toBeNull();
  });
});

describe('I-70 P7 summary and resume', { tags: ['critical'] }, () => {
  it('counts every outcome and the cases not run, a re-recorded case once', () => {
    const run = runOf([result('T-1', 'fail', 'x'), result('T-1', 'pass'), result('T-2', 'skip'), result('T-3', 'blocked', 'y')]);
    expect(summarize(run, IDS)).toEqual({ pass: 1, fail: 0, partial: 0, blocked: 1, skip: 1, notRun: 1, total: 4 });
  });

  it('keeps the summary the server wrote once the run has ended', () => {
    const stored = { pass: 3, fail: 1, partial: 0, blocked: 0, skip: 0, notRun: 0, total: 4 };
    expect(summarize(runOf([], { endedAt: 'z', summary: stored }), IDS)).toEqual(stored);
  });

  it('lists the failed, partial and blocked cases with their notes', () => {
    const run = runOf([result('T-1', 'pass'), result('T-2', 'fail', 'no button'), result('T-3', 'partial', 'half'), result('T-4', 'skip')]);
    expect(flaggedResults(run, IDS).map((r) => [r.test, r.note])).toEqual([['T-2', 'no button'], ['T-3', 'half']]);
  });

  it('resumes at the first case without a result, and at the start when all are done', () => {
    expect(firstOpen(runOf([result('T-1', 'pass'), result('T-2', 'skip')]), IDS)).toBe(2);
    expect(firstOpen(runOf([]), IDS)).toBe(0);
    expect(firstOpen(runOf(IDS.map((id) => result(id, 'pass'))), IDS)).toBe(0);
  });

  it('tones the strip by outcome and marks the current case', () => {
    const run = runOf([result('T-1', 'pass'), result('T-2', 'fail', 'x')]);
    const steps = caseSteps(run, IDS, new Map([['T-1', 'Login']]), 2);
    expect(steps.map((step) => step.outcome)).toEqual(['pass', 'fail', 'current', 'pending']);
    expect(steps[0]).toEqual({ id: 'T-1', outcome: 'pass', label: 'T-1 Login' });
    expect(steps[3]?.label).toBe('T-4');
  });
});

describe('I-70 P7 notes and keys', { tags: ['critical'] }, () => {
  it('requires a note for fail, partial and blocked only', () => {
    for (const status of ['fail', 'partial', 'blocked'] as const) expect(noteProblem(status, '  ')).toBe(`A note is required for ${status}.`);
    for (const status of ['pass', 'skip'] as const) expect(noteProblem(status, '')).toBeNull();
    expect(noteProblem('fail', 'seen')).toBeNull();
    expect(noteProblem('pass', 'x'.repeat(301))).toContain('300');
  });

  it('maps the keys to outcomes', () => {
    expect(['p', 'F', 's', 'b', 'm', 'x'].map(keyStatus)).toEqual(['pass', 'fail', 'skip', 'blocked', 'partial', null]);
  });
});

describe('I-70 P7 the runner state', { tags: ['critical'] }, () => {
  const at = (cursor: number, note = '', asked: RunnerState['asked'] = null): RunnerState => ({ cursor, note, asked });

  it('steps with Previous and Next and stops at both ends', () => {
    expect(runnerReducer(at(1), { type: 'step', by: 1, total: 4 }).cursor).toBe(2);
    expect(runnerReducer(at(1), { type: 'step', by: -1, total: 4 }).cursor).toBe(0);
    expect(runnerReducer(at(0), { type: 'step', by: -1, total: 4 }).cursor).toBe(0);
    expect(runnerReducer(at(3), { type: 'step', by: 1, total: 4 }).cursor).toBe(3);
  });

  it('jumps to a case and clears the note of the case it leaves', () => {
    expect(runnerReducer(at(0, 'draft', 'fail'), { type: 'goto', index: 2, total: 4 })).toEqual(at(2));
    expect(runnerReducer(at(0), { type: 'goto', index: -1, total: 4 }).cursor).toBe(0);
  });

  it('moves on after a record and clears the note', () => {
    expect(runnerReducer(at(1, 'seen'), { type: 'recorded', total: 4 })).toEqual(at(2));
    expect(runnerReducer(at(3, 'seen'), { type: 'recorded', total: 4 })).toEqual(at(3));
  });

  it('remembers the outcome that asked for a note until the note is typed', () => {
    const asked = runnerReducer(at(1), { type: 'ask', status: 'fail' });
    expect(asked.asked).toBe('fail');
    expect(runnerReducer(asked, { type: 'note', text: 'x' })).toEqual(at(1, 'x'));
  });
});

describe('I-70 P7 the posts', { tags: ['critical'] }, () => {
  it('posts plan-run-start on the plan item as the owner', async () => {
    const fetchMock = mockPosts();
    const sent = await planRunOp('rotp', 'I-1', { do: 'start' });
    expect(sent.result.status).toBe('applied');
    const post = await onlyPost(fetchMock);
    expect(post?.body).toMatchObject({ kind: 'plan-run-start', author: 'human', target: { c: 'items', id: 'I-1' } });
  });

  it('posts plan-run-record with the test, the status and the note', async () => {
    const fetchMock = mockPosts();
    await planRunOp('rotp', 'I-1', { do: 'record', test: 'T-2', status: 'fail', note: 'no button' });
    const post = await onlyPost(fetchMock);
    expect(post?.body).toMatchObject({ kind: 'plan-run-record', target: { c: 'items', id: 'I-1' }, test: 'T-2', status: 'fail', note: 'no button' });
  });

  it('posts plan-run-end', async () => {
    const fetchMock = mockPosts();
    await planRunOp('rotp', 'I-1', { do: 'end' });
    const post = await onlyPost(fetchMock);
    expect(post?.body).toMatchObject({ kind: 'plan-run-end', target: { c: 'items', id: 'I-1' } });
  });
});
