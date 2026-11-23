/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { controlsFor, modeSteps, pendingApprovals, spendLook } from '../../src/screens/overview/overlord/model';
import { isRunning } from '../../src/shell/OverlordChat/behavior/is-running';
import { toRows } from '../../src/shell/OverlordChat/behavior/to-rows';
import { STATE_STATUSES } from '../../src/screens/overview/overlord/overlord.constants';

const assistant = (index: number, content: unknown[]) => ({ index, event: { type: 'assistant', message: { content } } });
const user = (index: number, content: unknown) => ({ index, event: { type: 'user', message: { content } } });
const tool = (name: string, id = '', input?: unknown) => ({ type: 'tool_use', name, id, input });

describe('O1 the Overlord state words and tones', { tags: ['critical'] }, () => {
  it('has a plain label and a tone for every state', () => {
    expect(STATE_STATUSES.crashed).toMatchObject({ label: 'Crashed', tone: 'danger' });
    expect(STATE_STATUSES['waiting-approval'].tone).toBe('warning');
    expect(STATE_STATUSES.stopped.tone).toBe('neutral');
    expect(Object.keys(STATE_STATUSES)).toHaveLength(7);
  });
});

describe('O2 controls by state', { tags: ['critical'] }, () => {
  it('lets the Overlord restart unless it is starting', () => {
    expect(controlsFor('stopped')).toEqual({ restart: true });
    expect(controlsFor('starting')).toEqual({ restart: false });
    expect(controlsFor('idle')).toEqual({ restart: true });
    expect(controlsFor('crashed').restart).toBe(true);
    expect(controlsFor(undefined)).toEqual({ restart: false });
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
    expect(rows[1]).toMatchObject({ tools: [{ name: 'coord inbox' }, { name: 'coord board' }, { name: 'send_message' }] });
  });

  it('tells the owner from a wake, also when they arrive joined', () => {
    const rows = toRows([user(1, [{ type: 'text', text: 'WAKE inbox\n\nOwner (Argus Overlord tab): stop the build' }])]);
    expect(rows).toMatchObject([{ kind: 'wake', text: 'WAKE inbox' }, { kind: 'owner', text: 'stop the build' }]);
  });

  it('hides unknown lines and results with no call', () => {
    const rows = toRows([user(1, [{ type: 'tool_result', tool_use_id: 'x', content: 'ok' }]), { index: 2, event: 'raw text' }]);
    expect(rows).toEqual([]);
  });

  it('summarises a tool call and marks its result', () => {
    const rows = toRows([
      assistant(1, [tool('PowerShell', 't1', { command: 'Get-Date' }), tool('Read', 't2', { file_path: 'a.txt' }), tool('mcp__x__go', 't3', { n: 1, q: 'hello' })]),
      user(2, [{ type: 'tool_result', tool_use_id: 't1', content: 'ok' }, { type: 'tool_result', tool_use_id: 't2', is_error: true, content: [{ text: 'nope
more' }] }]),
    ]);
    expect(rows[0]).toMatchObject({
      kind: 'tools',
      tools: [
        { summary: 'Get-Date', status: 'done' },
        { summary: 'a.txt (error: nope)', status: 'failed' },
        { summary: 'hello', status: 'running' },
      ],
    });
  });

  it('cuts a long command at 120 characters', () => {
    const rows = toRows([assistant(1, [tool('Bash', 't', { command: 'x'.repeat(300) })])]);
    expect(rows[0]).toMatchObject({ tools: [{ summary: `${'x'.repeat(119)}...` }] });
  });

  it('shows thinking, a turn separator and rule lines', () => {
    const rows = toRows([
      assistant(1, [{ type: 'thinking', thinking: 'hmm' }]),
      { index: 2, event: { type: 'result', duration_ms: 2500 } },
      { index: 3, event: { type: 'user', argus: { source: 'rule' }, message: { content: 'Check the boards' } } },
    ]);
    expect(rows).toMatchObject([{ kind: 'thinking', text: 'hmm' }, { kind: 'turn', text: 'Turn finished in 2.5s' }, { kind: 'rule', text: 'Check the boards' }]);
  });

  it('keeps only the latest approval line per id', () => {
    const ask = (index: number, status: string) => ({ index, event: { type: 'argus', event: 'approval', id: 'a1', tool: 'PowerShell', summary: 'Get-Date', status, decidedBy: 'owner' } });
    const rows = toRows([ask(1, 'pending'), ask(2, 'allowed')]);
    expect(rows).toHaveLength(1);
    expect(rows[0]).toMatchObject({ kind: 'approval', text: 'Asked to run PowerShell: Get-Date, allowed by owner' });
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
    const base = { month: '2026-11', budgetUsd: 100 as number | null, warnAt: 0.8, leftUsd: 0, note: '' };
    expect(spendLook({ ...base, spentUsd: 23 })).toEqual({ ratio: 0.23, tone: 'success' });
    expect(spendLook({ ...base, spentUsd: 85 })?.tone).toBe('warning');
    expect(spendLook({ ...base, spentUsd: 130 })).toEqual({ ratio: 1, tone: 'danger' });
  });

  it('has no look when there is no cap', () => {
    expect(spendLook({ month: '2026-11', budgetUsd: null, warnAt: 0.8, leftUsd: null, note: '', spentUsd: 500 })).toBeNull();
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
