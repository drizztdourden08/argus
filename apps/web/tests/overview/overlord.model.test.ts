/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { controlsFor, modeSteps, pendingApprovals, spendLook } from '../../src/screens/overview/overlord/model';
import { isRunning } from '../../src/shell/OverlordChat/behavior/is-running';
import { toRows } from '../../src/shell/OverlordChat/behavior/to-rows';
import { STATE_STATUSES } from '../../src/screens/overview/overlord/overlord.constants';

const assistant = (index: number, content: unknown[]) => ({ index, event: { type: 'assistant', message: { content } } });
const user = (index: number, content: unknown) => ({ index, event: { type: 'user', message: { content } } });
const tool = (name: string) => ({ type: 'tool_use', name });

describe('O1 the Overlord state words and tones', { tags: ['critical'] }, () => {
  it('has a plain label and a tone for every state', () => {
    expect(STATE_STATUSES.crashed).toMatchObject({ label: 'Crashed', tone: 'danger' });
    expect(STATE_STATUSES['waiting-approval'].tone).toBe('warning');
    expect(STATE_STATUSES.stopped.tone).toBe('neutral');
    expect(Object.keys(STATE_STATUSES)).toHaveLength(7);
  });
});

describe('O2 controls by state', { tags: ['critical'] }, () => {
  it('lets a stopped Overlord start only and a running one restart or stop', () => {
    expect(controlsFor('stopped')).toEqual({ start: true, restart: false, stop: false });
    expect(controlsFor('idle')).toEqual({ start: false, restart: true, stop: true });
    expect(controlsFor('crashed').start).toBe(true);
    expect(controlsFor(undefined)).toEqual({ start: false, restart: false, stop: false });
  });

  it('takes chat only while the child runs', () => {
    expect(isRunning('turn')).toBe(true);
    expect(isRunning('stopped')).toBe(false);
    expect(isRunning(undefined)).toBe(false);
  });
});

describe('O3 transcript rows', { tags: ['critical'] }, () => {
  it('folds consecutive tool calls into one row and keeps text rows apart', () => {
    const rows = toRows([
      assistant(1, [{ type: 'text', text: 'Looking.' }, tool('coord inbox'), tool('coord board')]),
      assistant(2, [tool('send_message')]),
      assistant(3, [{ type: 'text', text: 'Done.' }]),
    ]);
    expect(rows.map((row) => row.kind)).toEqual(['assistant', 'tools', 'assistant']);
    expect(rows[1]).toMatchObject({ names: ['coord inbox', 'coord board', 'send_message'] });
  });

  it('tells the owner from a wake, also when they arrive joined', () => {
    const rows = toRows([user(1, [{ type: 'text', text: 'WAKE inbox\n\nOwner (Argus Overlord tab): stop the build' }])]);
    expect(rows).toMatchObject([{ kind: 'wake', text: 'WAKE inbox' }, { kind: 'owner', text: 'stop the build' }]);
  });

  it('hides tool results and unknown lines', () => {
    const rows = toRows([user(1, [{ type: 'tool_result', content: 'ok' }]), { index: 2, event: 'raw text' }]);
    expect(rows).toEqual([]);
  });
});

describe('O4 approvals and spend', { tags: ['critical'] }, () => {
  const ask = (id: string, at: string, status: 'pending' | 'allowed' = 'pending') => ({ id, at, tool: 'Bash', inputSummary: 'x', status });

  it('lists only pending approvals, oldest first, with the linked one first', () => {
    const all = [ask('b', '2026-11-14T10:05:00Z'), ask('a', '2026-11-14T10:00:00Z'), ask('c', '2026-11-14T10:09:00Z'), ask('d', '2026-11-14T09:00:00Z', 'allowed')];
    expect(pendingApprovals(all, null).map((a) => a.id)).toEqual(['a', 'b', 'c']);
    expect(pendingApprovals(all, 'c').map((a) => a.id)).toEqual(['c', 'a', 'b']);
  });

  it('turns warning at the warn level and danger at the budget', () => {
    const base = { month: '2026-11', budgetUsd: 100, warnAt: 0.8, leftUsd: 0, note: '' };
    expect(spendLook({ ...base, spentUsd: 23 })).toEqual({ ratio: 0.23, tone: 'success' });
    expect(spendLook({ ...base, spentUsd: 85 }).tone).toBe('warning');
    expect(spendLook({ ...base, spentUsd: 130 })).toEqual({ ratio: 1, tone: 'danger' });
  });
});

describe('O9 mode steps', { tags: ['critical'] }, () => {
  const allowed = (mode: 'off' | 'shadow' | 'live') => modeSteps(mode).filter((step) => step.allowed).map((step) => step.mode);

  it('allows only the legal steps from each mode', () => {
    expect(allowed('off')).toEqual(['off', 'shadow']);
    expect(allowed('shadow')).toEqual(['off', 'shadow', 'live']);
    expect(allowed('live')).toEqual(['off', 'live']);
  });

  it('says why a step is refused', () => {
    expect(modeSteps('off')[2]).toMatchObject({ mode: 'live', allowed: false, reason: 'Start shadow first' });
    expect(modeSteps('live')[1]).toMatchObject({ mode: 'shadow', allowed: false, reason: 'Go to off first' });
    expect(modeSteps('shadow').find((step) => step.current)?.mode).toBe('shadow');
  });
});
