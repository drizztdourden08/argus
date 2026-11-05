/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { runLine } from '../../src/data/runs';
import type { RunEntry } from '../../src/data/runs';
import {
  countsText, DEFAULT_FILTERS, noteProblem, resolveLaunch, sortDone, testsView, visibleTests,
} from '../../src/screens/board/tabs/tests/model';
import { runCounts } from '../../src/screens/board/tabs/tests/model/run-counts';
import { runViews } from '../../src/screens/board/tabs/tests/model/run-views';
import { tally } from '../../src/screens/board/tabs/tests/model/tally';
import type { ItemDoc, LaunchDefDoc, LauncherDoc, TestDoc } from '../../src/screens/board/tabs/tests/model';
import { asRunEntry, defDoc, doneDoc, itemDoc, launcherDoc, runDoc, testDoc, testRunDoc } from './fixtures';

const tests = (...docs: object[]) => docs as TestDoc[];
const items = (...docs: object[]) => docs as ItemDoc[];

describe('K1 counts', { tags: ['critical'] }, () => {
  it('counts to run apart from the results, moved counts as pass, archived tests do not count', () => {
    const rows = tests(
      testDoc('T-1'),
      testDoc('T-2', { archivedAt: 'then' }),
      doneDoc('T-3', 'pass', 'ok'),
      doneDoc('T-4', 'moved', 'elsewhere'),
      doneDoc('T-5', 'fail', 'no'),
      doneDoc('T-6', 'blocked', 'stuck'),
    );
    expect(tally(rows)).toEqual({ run: 1, pass: 2, fail: 1, partial: 0, blocked: 1 });
    expect(countsText(tally(rows))).toBe('1 to run · 2 pass · 1 fail · 1 blocked');
    expect(countsText(tally([]))).toBe('no tests yet');
  });

  it('visibleTests leaves out the archived and the deleted', () => {
    const rows = tests(testDoc('T-1'), testDoc('T-2', { archivedAt: 'then' }), testDoc('T-3', { deletedAt: 'then' }));
    expect(visibleTests(rows).map((test) => test.id)).toEqual(['T-1']);
  });
});

const plan = (cases: string[]) => itemDoc('I-5', { title: 'Plan five', subject: null, type: 'test-plan', stage: 'ready', fields: { cases } });
const run = (id: string, stage: string, extra: object = {}) => testRunDoc(id, 'I-5', { stage, ...extra });
const input = (docs: ItemDoc[]) => ({ items: docs, tests: [], launchers: [], defs: [], filters: DEFAULT_FILTERS });

describe('K1 the counts of a run', { tags: ['critical'] }, () => {
  it('counts the last status of each case of the plan, and every case without a result to run', () => {
    const results = [{ test: 'T-1', status: 'fail' }, { test: 'T-1', status: 'pass' }, { test: 'T-2', status: 'blocked' }, { test: 'T-9', status: 'pass' }];
    expect(runCounts(results, ['T-1', 'T-2', 'T-3'])).toEqual({ run: 1, pass: 1, fail: 0, partial: 0, blocked: 1 });
    expect(runCounts(undefined, ['T-1'])).toEqual({ run: 1, pass: 0, fail: 0, partial: 0, blocked: 0 });
  });
});

describe('K1 the runs of the board', { tags: ['critical'] }, () => {
  const docs = items(plan(['T-1']), run('I-11', 'draft'), run('I-12', 'running'), run('I-13', 'ready', { assignee: 'owner' }), run('I-14', 'passed'), run('I-15', 'abandoned'), run('I-16', 'running', { deletedAt: 'x' }));

  it('puts the handed run first, then the others newest first, and the drafts last, leaving out the deleted', () => {
    expect(runViews(input(docs)).map((view) => view.item.id)).toEqual(['I-13', 'I-15', 'I-14', 'I-12', 'I-11']);
  });

  it('keeps the closed runs apart from the open ones and counts them', () => {
    const view = testsView({ ...input(docs), tests: [], prefs: { group: 'run', order: 'rank', showPassed: true, showClosed: false }, conflicted: new Set() });
    expect(view.runs.map((open) => open.item.id)).toEqual(['I-13', 'I-12', 'I-11']);
    expect(view.closed.map((closed) => closed.item.id)).toEqual(['I-15', 'I-14']);
    expect(view.closedCount).toBe(2);
  });

  it('opens a run that is handed or running, and folds a draft', () => {
    const open = Object.fromEntries(runViews(input(docs)).map((view) => [view.item.id, view.open]));
    expect(open).toMatchObject({ 'I-13': true, 'I-12': true, 'I-11': false });
  });

  it('does not count a closed run as handed', () => {
    const closed = items(plan([]), run('I-20', 'passed', { assignee: 'owner' }));
    expect(runViews(input(closed))[0]?.handed).toBe(false);
  });
});

describe('K1 the launch of a run', { tags: ['critical'] }, () => {
  const launchers = [{ ...launcherDoc('b1'), id: 'b1' }, { ...launcherDoc('gone', { retiredAt: 'then', retiredBy: 'lead', retireNote: 'replaced' }), id: 'gone' }] as LauncherDoc[];
  const defs = [{ ...defDoc('b1'), id: 'launcher-b1' }] as LaunchDefDoc[];

  it('reads the command live from the linked launcher and finds its run definition', () => {
    const launch = resolveLaunch('b1', launchers, defs);
    expect(launch).toMatchObject({ state: 'linked', name: 'b1', cmd: 'rotp launch b1' });
    expect(launch.def?.name).toBe('launcher-b1');
  });

  it('says a retired launcher was retired and a missing one is missing, with no command', () => {
    expect(resolveLaunch('gone', launchers, defs)).toMatchObject({ state: 'retired', cmd: '', gone: 'launcher gone was retired by lead (replaced)' });
    expect(resolveLaunch('nope', launchers, defs)).toMatchObject({ state: 'missing', cmd: '' });
  });

  it('has nothing to launch for a run without a build', () => {
    expect(resolveLaunch(null, launchers, defs).state).toBe('none');
  });
});

describe('K1 the status line of a run', { tags: ['critical'] }, () => {
  const run = (extra: object) => asRunEntry({ ...runDoc('R-1', 'launcher-b1'), ...extra });

  it('says queued, then launching until the first output line, then running with the line', () => {
    expect(runLine(run({ status: 'queued' })).label).toBe('Queued, waiting for a free slot');
    expect(runLine(run({ status: 'running', endedAt: null })).label).toBe('Launching...');
    expect(runLine(run({ status: 'running', endedAt: null, lastLine: 'ready' }))).toMatchObject({ label: 'running', last: 'ready', tone: 'success' });
  });

  it('says how a run ended', () => {
    expect(runLine(run({}))).toMatchObject({ label: 'exited 0', tone: 'info' });
    expect(runLine(run({ status: 'stopped', exitCode: 1 })).label).toBe('stopped');
    expect(runLine(run({ status: 'failed', exitCode: 3 }))).toMatchObject({ label: 'failed (exit 3)', tone: 'danger' });
    expect(runLine(run({ status: 'failed', exitCode: null, error: 'spawn ENOENT' })).label).toBe('failed (spawn ENOENT)');
  });

  it('strips colour codes, folds spaces and cuts a long line with three dots', () => {
    const coloured = `${String.fromCharCode(27)}[31mred${String.fromCharCode(27)}[0m    text`;
    expect(runLine(run({ lastLine: coloured })).last).toBe('red text');
    const cut = runLine(run({ lastLine: 'x'.repeat(300) })).last;
    expect(cut).toHaveLength(120);
    expect(cut.endsWith('...')).toBe(true);
  });
});

describe('K1 the done order and the note rule', { tags: ['critical'] }, () => {
  it('lists the newest result first', () => {
    const rows = tests(
      doneDoc('T-1', 'pass', 'a', { result: { status: 'pass', text: 'a', by: 'human', at: '2026-10-26T08:00:00.000Z' } }),
      doneDoc('T-2', 'pass', 'b', { result: { status: 'pass', text: 'b', by: 'human', at: '2026-10-26T09:00:00.000Z' } }),
    );
    expect(sortDone(rows).map((test) => test.id)).toEqual(['T-2', 'T-1']);
  });

  it('asks for three non-space characters on fail, partial and blocked, never on pass', () => {
    expect(noteProblem('pass', '')).toBeNull();
    expect(noteProblem('fail', '  a b ')).toMatch(/three characters/);
    expect(noteProblem('blocked', 'abc')).toBeNull();
    expect(noteProblem('partial', 'x'.repeat(50001))).toBe('This text is 50001 characters and one post takes 50000 at most: shorten it or post it in two parts.');
  });
});
