/* @layer renderer-app @kind test */
import { act, cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import type { RegistryRow } from '../../src/data/types';
import { chipLook } from '../../src/screens/board/RegisteredStrip/behavior/chip-look';
import { RegisteredStrip } from '../../src/screens/board/RegisteredStrip';
import { ChipTip } from '../../src/screens/board/RegisteredStrip/sub-components/ChipTip';
import { createFakeSources } from '../helpers/fake-source';
import { changeBoard, seedBoard } from '../helpers/seed-board';

afterEach(cleanup);

const row = (over: Partial<RegistryRow> & { thread: string }): RegistryRow => ({
  id: over.thread,
  project: 'rotp',
  session: over.thread,
  role: 'thread',
  state: 'listening',
  ...over,
});

const minutesAgo = (minutes: number): string => new Date(Date.now() - minutes * 60000).toISOString();

const faulted = (thread: string, reason: string, over: Partial<RegistryRow> = {}): RegistryRow =>
  row({ thread, state: 'fault', fault: { since: minutesAgo(5), reason }, ...over });

const waking = (thread: string): RegistryRow => row({ thread, state: 'waking', pendingWake: { since: minutesAgo(2) } });

const mount = () => {
  const fake = createFakeSources();
  render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <RegisteredStrip boardKey="rotp" />
    </DataProvider>,
  );
  return fake.sources;
};

const seeded = async (docs: Record<string, Record<string, object>>) => {
  const sources = mount();
  await waitFor(() => expect(sources).toHaveLength(1));
  act(() => seedBoard(sources[0], docs));
  return sources;
};

const registry = (...rows: RegistryRow[]) => Object.fromEntries(rows.map((entry) => [entry.id, entry]));

describe('the Registered strip', () => {
  it('is hidden until a thread is registered, and follows the registry stream', async () => {
    const sources = mount();
    await waitFor(() => expect(sources).toHaveLength(1));
    expect(sources[0]?.url).toContain('registry');
    act(() => seedBoard(sources[0], { registry: {} }));
    expect(screen.queryByRole('status', { name: 'Registered threads' })).toBeNull();
    act(() => changeBoard(sources[0], 'registry', 'lead', row({ thread: 'lead', role: 'coordinator' })));
    expect(screen.getByRole('status', { name: 'Registered threads' })).toBeTruthy();
    expect(screen.getByText('Registered 1')).toBeTruthy();
  });

  it('says how many are registered, waking and faulted, with one chip per thread, worst state first', async () => {
    await seeded({
      registry: registry(
        row({ thread: 'lead', role: 'coordinator' }),
        waking('beta'),
        faulted('gamma', 'no-listener'),
        row({ thread: 'delta', state: 'idle' }),
      ),
    });
    expect(screen.getByText('Registered 4')).toBeTruthy();
    expect(screen.getByText('1 waking')).toBeTruthy();
    expect(screen.getByText('1 fault')).toBeTruthy();
    const names = screen.getAllByText(/^(lead|beta|gamma|delta)$/).map((node) => node.textContent);
    expect(names).toEqual(['gamma', 'beta', 'lead', 'delta']);
    expect(screen.getByText('wake failed')).toBeTruthy();
    expect(screen.getByText('waking')).toBeTruthy();
    expect(screen.getByText('not listening')).toBeTruthy();
  });

  it('has no control to clear a fault', async () => {
    await seeded({ registry: registry(faulted('gamma', 'no-reaction')) });
    expect(screen.queryByRole('button', { name: /clear/i })).toBeNull();
  });
});

describe('the agents and the live state of the strip', () => {
  it('counts the agents working per thread, and the stalled ones apart', async () => {
    const agent = (id: string, over: object) => [id, { thread: 'lead', status: 'working', updatedAt: minutesAgo(1), ...over }] as const;
    await seeded({
      registry: registry(row({ thread: 'lead', role: 'coordinator' })),
      agents: Object.fromEntries([
        agent('A-1', {}),
        agent('A-2', { updatedAt: minutesAgo(120) }),
        agent('A-3', { live: { stalled: true } }),
        agent('A-4', { status: 'done' }),
        agent('A-5', { archivedAt: minutesAgo(3) }),
      ]),
    });
    expect(screen.getByText('+1 agent working')).toBeTruthy();
    expect(screen.getByText('2 stalled')).toBeTruthy();
  });

  it('updates live: a fault that clears and a thread that returns to listening redraw the strip', async () => {
    const sources = await seeded({ registry: registry(faulted('gamma', 'no-listener'), waking('beta')) });
    expect(screen.getByText('1 fault')).toBeTruthy();
    act(() => changeBoard(sources[0], 'registry', 'gamma', row({ thread: 'gamma' })));
    act(() => changeBoard(sources[0], 'registry', 'beta', row({ thread: 'beta' })));
    expect(screen.queryByText('1 fault')).toBeNull();
    expect(screen.queryByText('1 waking')).toBeNull();
    expect(screen.getByText('Registered 2')).toBeTruthy();
  });
});

describe('the tooltip of a chip', () => {
  const tip = (registryRow: RegistryRow) => {
    const { container } = render(<ChipTip row={registryRow} />);
    return container.textContent;
  };

  it('a fault says why and that only the Overlord can clear it, and a thread is told it has been notified', () => {
    const text = tip(faulted('gamma', 'no-listener'));
    expect(text).toContain('Wake failed: no listener since');
    expect(text).toContain('Only the Overlord can clear this; it has been notified.');
  });

  it('the fault of the Overlord does not say it has been notified', () => {
    const text = tip(faulted('lead', 'no-reaction', { project: 'portfolio', role: 'coordinator' }));
    expect(text).toContain('Only the Overlord can clear this.');
    expect(text).not.toContain('notified');
  });

  it('a closed board fault says the thread has no open board tab', () => {
    const text = tip(faulted('gamma', 'board-closed'));
    expect(text).toContain('this thread has no open board tab');
    expect(text).toContain('it clears itself when the board opens.');
  });

  it('a waking, busy, listening and idle chip each say what the thread is doing', () => {
    expect(tip(waking('beta'))).toContain('beta: waking up, a wake was sent');
    expect(tip(row({ thread: 'beta', state: 'busy' }))).toContain('in a turn right now');
    expect(tip(row({ thread: 'beta', state: 'listening', listenerLabel: 'Beta' }))).toContain('beta: listening (Beta)');
    expect(tip(row({ thread: 'beta', state: 'idle', deaf: true }))).toContain('The session is active');
    expect(tip(row({ thread: 'lead', role: 'coordinator', state: 'listening' }))).toContain('lead (lead): listening');
  });

  it('a thread whose board tab is closed says so, one whose board is open does not', () => {
    expect(tip(row({ thread: 'beta', boardRequired: true, boardOpen: false }))).toContain('Its board tab is not open');
    expect(tip(row({ thread: 'beta', boardRequired: true, boardOpen: true }))).not.toContain('board tab');
  });
});

describe('the look of a chip', () => {
  it('maps every state to a tone, a mark and a word', () => {
    expect(chipLook(row({ thread: 'a' }))).toMatchObject({ tone: 'success', mark: 'dot', word: '' });
    expect(chipLook(row({ thread: 'a', state: 'busy' }))).toMatchObject({ tone: 'info', word: 'working' });
    expect(chipLook(row({ thread: 'a', state: 'waking' }))).toMatchObject({ tone: 'warning', mark: 'spin', word: 'waking' });
    expect(chipLook(row({ thread: 'a', state: 'idle' }))).toMatchObject({ tone: 'neutral', word: 'not listening' });
    expect(chipLook(row({ thread: 'a', state: 'idle', deaf: true }))).toMatchObject({ tone: 'warning', mark: 'warn' });
    expect(chipLook(row({ thread: 'a', state: 'fault', fault: { since: 'x', reason: 'no-listener' } }))).toMatchObject({
      tone: 'danger',
      mark: 'warn',
      word: 'wake failed',
      faulted: true,
    });
  });

  it('adds board closed to a thread that needs its board and has none open', () => {
    expect(chipLook(row({ thread: 'a', boardRequired: true, boardOpen: false })).word).toBe('board closed');
    expect(chipLook(row({ thread: 'a', state: 'idle', boardRequired: true, boardOpen: false })).word).toBe('not listening · board closed');
  });
});
