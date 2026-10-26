/* @layer renderer-app @kind test */
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from '../../src/data/api';
import { killProc, killRule } from '../../src/data/procs';
import type { ProcItem } from '../../src/screens/overview/processes/model';
import {
  askStillValid, filterProcs, groupCap, isOverCap, matchText, needsLook, procLabel, ruleGroups, ruleNames, ruleTip, rulesOf, sortProcs, staleOf, statusTip, toProcRow,
} from '../../src/screens/overview/processes/model';
import { killError } from '../../src/screens/overview/processes/model/kill-error';
import { killOne } from '../../src/screens/overview/processes/model/kill-one';
import { killStale } from '../../src/screens/overview/processes/model/kill-stale';
import { killedCount } from '../../src/screens/overview/processes/model/killed-count';

vi.mock('../../src/data/procs', () => ({ killProc: vi.fn(), killRule: vi.fn() }));

const proc = (id: string, extra: Partial<ProcItem> = {}): ProcItem => ({ id, pid: 100, name: 'node', rule: 'node-other', memMB: 100, status: 'ok', ...extra });

const refusal = (status: number): ApiError => new ApiError(status, { code: 'x', message: 'no' });

beforeEach(() => {
  vi.mocked(killProc).mockReset();
  vi.mocked(killRule).mockReset();
});

describe('what needs a look, and the order of the rows', () => {
  it('needs a look when the status is anything but ok; a row with no status does not', () => {
    expect(needsLook(proc('a', { status: 'ok' }))).toBe(false);
    expect(needsLook(proc('a', { status: 'hog' }))).toBe(true);
    expect(needsLook({})).toBe(false);
  });

  it('puts those that need a look first, then memory descending, then name, then pid', () => {
    const rows = [
      proc('ok-big', { memMB: 900, name: 'b' }),
      proc('look-small', { memMB: 10, status: 'stale' }),
      proc('look-big', { memMB: 500, status: 'hog' }),
      proc('ok-same-b', { memMB: 50, name: 'b', pid: 2 }),
      proc('ok-same-a', { memMB: 50, name: 'a', pid: 9 }),
    ];
    expect(sortProcs(rows).map((row) => row.id)).toEqual(['look-big', 'look-small', 'ok-big', 'ok-same-a', 'ok-same-b']);
  });

  it('labels a process by name and pid, with fallbacks', () => {
    expect(procLabel(proc('a', { name: 'vite', pid: 4812 }))).toBe('vite 4812');
    expect(procLabel({})).toBe('process ?');
  });
});

describe('the rule strip groups', () => {
  it('takes the groups of the meta doc and drops one without a rule', () => {
    const meta = { byRule: [{ rule: 'a', count: 1, memMB: 5 }, { rule: '', count: 2 }] };
    expect(ruleGroups(meta, []).map((group) => group.rule)).toEqual(['a']);
  });

  it('without meta counts the rows by rule, the biggest memory first, and skips rows with no rule', () => {
    const rows = [proc('1', { rule: 'a', memMB: 10 }), proc('2', { rule: 'b', memMB: 300 }), proc('3', { rule: 'a', memMB: 20 }), proc('4', { rule: null })];
    expect(ruleGroups(null, rows)).toEqual([{ rule: 'b', count: 1, memMB: 300 }, { rule: 'a', count: 2, memMB: 30 }]);
  });

  it('lists every rule name once, sorted, from the rules, the groups and the rows', () => {
    const meta = { rules: [{ name: 'zed' }, { name: 'alpha' }], byRule: [{ rule: 'mid' }] };
    expect(ruleNames(meta, [proc('1', { rule: 'alpha' }), proc('2', { rule: 'extra' })])).toEqual(['alpha', 'extra', 'mid', 'zed']);
  });

  it('reads the rules of the meta and ignores a nameless one', () => {
    expect(rulesOf({ rules: [{ name: 'a' }, { name: '' }] }).map((rule) => rule.name)).toEqual(['a']);
    expect(rulesOf(null)).toEqual([]);
  });

  it('is over its cap only when the cap is positive and the memory is past it', () => {
    expect(isOverCap(500, 400)).toBe(true);
    expect(isOverCap(400, 400)).toBe(false);
    expect(isOverCap(500, null)).toBe(false);
    expect(isOverCap(500, 0)).toBe(false);
  });

  it('takes the cap from the rule first, then from the group', () => {
    expect(groupCap({ rule: 'a', maxGroupMemMB: 100 }, { name: 'a', maxGroupMemMB: 200 })).toBe(200);
    expect(groupCap({ rule: 'a', maxGroupMemMB: 100 }, undefined)).toBe(100);
    expect(groupCap({ rule: 'a' }, undefined)).toBeNull();
  });

  it('writes the tip with the count, the memory, the cap and whether it is over', () => {
    expect(ruleTip({ rule: 'node', count: 3, memMB: 2048 }, 1024, true)).toBe('node: 3 process(es), 2.0 GB, group cap 1.0 GB (over)');
    expect(ruleTip({ rule: 'node', count: 1, memMB: 100 }, null, false)).toBe('node: 1 process(es), 100 MB');
  });

  it('prints the match of a rule as name, cmd and path patterns', () => {
    expect(matchText({ name: '^node', cmd: 'vite' })).toBe('name: ^node, cmd: vite');
    expect(matchText(undefined)).toBe('');
  });
});

describe('the filters and the stale ones', () => {
  const rows = [
    proc('1', { rule: 'a', status: 'ok' }),
    proc('2', { rule: 'a', status: 'stale' }),
    proc('3', { rule: 'b', status: 'stale' }),
  ];

  it('narrows by status and by rule, and both combine', () => {
    expect(filterProcs(rows, { status: 'all', rule: '' })).toHaveLength(3);
    expect(filterProcs(rows, { status: 'look', rule: '' }).map((row) => row.id)).toEqual(['2', '3']);
    expect(filterProcs(rows, { status: 'all', rule: 'a' }).map((row) => row.id)).toEqual(['1', '2']);
    expect(filterProcs(rows, { status: 'look', rule: 'a' }).map((row) => row.id)).toEqual(['2']);
  });

  it('counts as stale only a killable, unprotected, stale process of the rule', () => {
    const list = [
      proc('1', { rule: 'a', status: 'stale', killable: true }),
      proc('2', { rule: 'a', status: 'stale', killable: true, protected: true }),
      proc('3', { rule: 'a', status: 'stale', killable: false }),
      proc('4', { rule: 'a', status: 'ok', killable: true }),
      proc('5', { rule: 'b', status: 'stale', killable: true }),
    ];
    expect(staleOf(list, 'a').map((row) => row.id)).toEqual(['1']);
  });

  it('forgets a question about a process that went away, or a rule with nothing stale left', () => {
    const list = [proc('1', { rule: 'a', status: 'stale', killable: true })];
    expect(askStillValid({ kind: 'proc', id: '1' }, list)).toEqual({ kind: 'proc', id: '1' });
    expect(askStillValid({ kind: 'proc', id: '2' }, list)).toBeNull();
    expect(askStillValid({ kind: 'rule', rule: 'a' }, list)).toEqual({ kind: 'rule', rule: 'a' });
    expect(askStillValid({ kind: 'rule', rule: 'b' }, list)).toBeNull();
    expect(askStillValid(null, list)).toBeNull();
  });
});

describe('the table rows and the status tip', () => {
  it('keeps a row with only an id: dashes and nulls, no crash', () => {
    expect(toProcRow({ id: 'x' }, (key) => key)).toEqual({
      id: 'x', rule: '-', name: '?', pid: null, project: '', memMB: null, cpuPct: null, ageMin: null, idleMin: null, status: '', cmd: '', kill: '', item: { id: 'x' },
    });
  });

  it('shows the project by its name', () => {
    expect(toProcRow(proc('x', { project: 'rotp' }), () => 'Relic of the Past').project).toBe('Relic of the Past');
  });

  it('says what the status means and lists the flags, one per line', () => {
    expect(statusTip(proc('x', { status: 'hog', flags: ['group'] }))).toBe('Uses more memory than its rule allows.\ngroup');
    expect(statusTip(proc('x', { status: 'weird' }))).toBe('weird');
  });
});

describe('the sentences of a kill', () => {
  it('counts what the server says it killed: a list, a number, or the fallback', () => {
    expect(killedCount({ killed: [1, 2, 3] }, 1)).toBe(3);
    expect(killedCount({ killed: 4 }, 1)).toBe(4);
    expect(killedCount({ killed: true }, 1)).toBe(1);
    expect(killedCount(null, 2)).toBe(2);
  });

  it('gives 403, 404 and 409 each their own sentence, an unreachable dashboard its own, and anything else the message', () => {
    const said = { denied: 'denied', gone: 'gone', changed: 'changed' };
    expect(killError(refusal(403), said)).toBe('denied');
    expect(killError(refusal(404), said)).toBe('gone');
    expect(killError(refusal(409), said)).toBe('changed');
    expect(killError(new ApiError(0, { code: 'unavailable', message: 'down' }), said)).toBe('Dashboard unreachable, retrying.');
    expect(killError(refusal(500), said)).toBe('Kill failed: no.');
    expect(killError(new Error('boom'), said)).toBe('Kill failed: boom.');
  });
});

describe('killOne', () => {
  const vite = proc('4812-1', { pid: 4812, name: 'vite', startedAt: '2026-10-26T10:00:00.000Z' });

  it('posts the pid, the startedAt exactly as sampled and tree false, and says it killed the process', async () => {
    vi.mocked(killProc).mockResolvedValue({ killed: true });
    const said = await killOne(vite, false);
    expect(killProc).toHaveBeenCalledWith({ pid: 4812, startedAt: '2026-10-26T10:00:00.000Z', tree: false });
    expect(said).toEqual({ variant: 'success', message: 'Killed vite 4812.' });
  });

  it('posts tree true for Kill with children and reports a count the server gives back', async () => {
    vi.mocked(killProc).mockResolvedValue({ killed: [4812, 4813, 4814] });
    const said = await killOne(vite, true);
    expect(killProc).toHaveBeenCalledWith({ pid: 4812, startedAt: '2026-10-26T10:00:00.000Z', tree: true });
    expect(said.message).toBe('Killed vite 4812 and its children (3 processes).');
  });

  it('says a protected process is not killed (403), a gone one is gone (404), a reused pid is not the one sampled (409)', async () => {
    vi.mocked(killProc).mockRejectedValueOnce(refusal(403));
    expect((await killOne(vite, false)).message).toBe('Argus will not kill vite 4812: it is protected, or its rule does not allow kills.');
    vi.mocked(killProc).mockRejectedValueOnce(refusal(404));
    expect((await killOne(vite, false)).message).toBe('vite 4812 is already gone, nothing was killed.');
    vi.mocked(killProc).mockRejectedValueOnce(refusal(409));
    const changed = await killOne(vite, false);
    expect(changed.variant).toBe('danger');
    expect(changed.message).toContain('is not the process that was sampled');
  });
});

describe('killStale', () => {
  it('posts the rule with idleOnly and counts what was killed', async () => {
    vi.mocked(killRule).mockResolvedValue({ killed: [1, 2] });
    const said = await killStale('node-dev-server', 5);
    expect(killRule).toHaveBeenCalledWith('node-dev-server');
    expect(said).toEqual({ variant: 'success', message: 'Killed 2 stale node-dev-server processes.' });
  });

  it('falls back on the count it showed and says one process in the singular', async () => {
    vi.mocked(killRule).mockResolvedValue({});
    expect((await killStale('python', 1)).message).toBe('Killed 1 stale python process.');
  });

  it('gives 403, 404 and 409 plain sentences too', async () => {
    vi.mocked(killRule).mockRejectedValueOnce(refusal(403));
    expect((await killStale('shell-core', 1)).message).toBe('Argus will not kill the shell-core processes: the rule does not allow kills.');
    vi.mocked(killRule).mockRejectedValueOnce(refusal(404));
    expect((await killStale('shell-core', 1)).message).toBe('No stale shell-core process is left, nothing was killed.');
    vi.mocked(killRule).mockRejectedValueOnce(refusal(409));
    expect((await killStale('shell-core', 1)).message).toContain('changed since they were sampled');
  });
});
