/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { runLine } from '../../src/data/runs';
import type { RunEntry } from '../../src/data/runs';
import {
  closedBundles, countsText, groupBundles, noteProblem, resolveLaunch, sortDone, visibleTests,
} from '../../src/screens/board/tabs/tests/model';
import { bundleState } from '../../src/screens/board/shared';
import { tally } from '../../src/screens/board/tabs/tests/model/tally';
import type { BuildDoc, LaunchDefDoc, LauncherDoc, TestDoc } from '../../src/screens/board/tabs/tests/model';
import { buildDoc, defDoc, doneDoc, launcherDoc, runDoc, testDoc } from './fixtures';

const tests = (...docs: object[]) => docs as TestDoc[];
const builds = (...docs: object[]) => docs.map((doc) => ({ ...doc, id: (doc as { tag: string }).tag })) as BuildDoc[];

describe('K1 bundle states', { tags: ['critical'] }, () => {
  it('reads closed from closedAt, draft from status, anything else as ready', () => {
    expect(bundleState(null)).toBe('ready');
    expect(bundleState(buildDoc('b', { status: 'draft' }))).toBe('draft');
    expect(bundleState(buildDoc('b', { closedAt: '2026-10-26T00:00:00.000Z', status: 'draft' }))).toBe('closed');
  });
});

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

describe('K1 grouping into bundles', { tags: ['critical'] }, () => {
  it('puts the handed bundle first, then the ones with tests to run (newest tag first), the finished ones, then no bundle', () => {
    const rows = tests(
      testDoc('T-1', { build: 'bundle-2' }),
      doneDoc('T-2', 'pass', 'ok', { build: 'bundle-3' }),
      testDoc('T-3', { build: 'bundle-10' }),
      testDoc('T-4', { build: undefined }),
      doneDoc('T-5', 'pass', 'ok', { build: 'bundle-1' }),
    );
    const groups = groupBundles({ tests: rows, builds: builds(buildDoc('bundle-1'), buildDoc('bundle-2'), buildDoc('bundle-3'), buildDoc('bundle-10')), handed: 'bundle-1' });
    expect(groups.map((group) => group.tag)).toEqual(['bundle-1', 'bundle-10', 'bundle-2', 'bundle-3', '(no build)']);
  });

  it('shows a declared bundle before it has a test, and the handed tag even without a record', () => {
    const groups = groupBundles({ tests: [], builds: builds(buildDoc('bundle-4')), handed: 'bundle-9' });
    expect(groups.map((group) => group.tag).sort()).toEqual(['bundle-4', 'bundle-9']);
    expect(groups.find((group) => group.tag === 'bundle-9')?.build).toBeNull();
  });

  it('keeps closed bundles and their tests out, and lists them newest close first', () => {
    const records = builds(
      buildDoc('old', { closedAt: '2026-10-01T00:00:00.000Z' }),
      buildDoc('newer', { closedAt: '2026-10-20T00:00:00.000Z' }),
      buildDoc('open'),
    );
    const rows = tests(testDoc('T-1', { build: 'old' }), testDoc('T-2', { build: 'open' }));
    const groups = groupBundles({ tests: rows, builds: records, handed: null });
    expect(groups.map((group) => group.tag)).toEqual(['open']);
    expect(groups[0]?.tests.map((test) => test.id)).toEqual(['T-2']);
    expect(closedBundles(rows, records).map((group) => [group.tag, group.tests.length])).toEqual([['newer', 0], ['old', 1]]);
  });

  it('opens a bundle that is handed or has tests to run, and folds a finished one', () => {
    const rows = tests(testDoc('T-1', { build: 'a' }), doneDoc('T-2', 'pass', 'ok', { build: 'b' }), doneDoc('T-3', 'pass', 'ok', { build: 'c' }));
    const groups = groupBundles({ tests: rows, builds: builds(buildDoc('a'), buildDoc('b'), buildDoc('c')), handed: 'c' });
    expect(Object.fromEntries(groups.map((group) => [group.tag, group.open]))).toEqual({ a: true, b: false, c: true });
  });
});

describe('K1 the launch of a bundle', { tags: ['critical'] }, () => {
  const launchers = [{ ...launcherDoc('b1'), id: 'b1' }, { ...launcherDoc('gone', { retiredAt: 'then', retiredBy: 'lead', retireNote: 'replaced' }), id: 'gone' }] as LauncherDoc[];
  const defs = [{ ...defDoc('b1'), id: 'launcher-b1' }, { ...defDoc('x', { launcher: undefined, bundle: 'old', name: 'bundle-old' }), id: 'bundle-old' }] as LaunchDefDoc[];
  const build = (extra: object, tag = 't') => builds(buildDoc(tag, extra))[0] ?? null;

  it('reads the command live from the linked launcher and finds its run definition', () => {
    const launch = resolveLaunch(build({ launcher: 'b1' }), launchers, defs);
    expect(launch).toMatchObject({ state: 'linked', name: 'b1', cmd: 'rotp launch b1' });
    expect(launch.def?.name).toBe('launcher-b1');
  });

  it('says a retired launcher was retired and a missing one is missing, with no command', () => {
    expect(resolveLaunch(build({ launcher: 'gone' }), launchers, defs)).toMatchObject({ state: 'retired', cmd: '', gone: 'launcher gone was retired by lead (replaced)' });
    expect(resolveLaunch(build({ launcher: 'nope' }), launchers, defs)).toMatchObject({ state: 'missing', cmd: '' });
  });

  it('plays a bundle written before launchers with its own command and its bundle definition', () => {
    const launch = resolveLaunch(build({ cmd: 'rotp old' }, 'old'), launchers, defs);
    expect(launch).toMatchObject({ state: 'legacy', cmd: 'rotp old' });
    expect(launch.def?.name).toBe('bundle-old');
    expect(resolveLaunch(build({}), launchers, defs).state).toBe('none');
  });
});

describe('K1 the status line of a run', { tags: ['critical'] }, () => {
  const run = (extra: object) => ({ ...runDoc('R-1', 'launcher-b1'), ...extra }) as RunEntry;

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
