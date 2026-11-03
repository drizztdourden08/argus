/* @layer renderer-app @kind test */
import { act, cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import type { RegistryRow } from '../../src/data/types';
import { chipLook } from '../../src/screens/board/RegisteredStrip/behavior/chip-look';
import { RegisteredStrip } from '../../src/screens/board/RegisteredStrip';
import { createFakeSources } from '../helpers/fake-source';
import { changeBoard, seedBoard } from '../helpers/seed-board';
import { waitForStream } from '../helpers/wait-for-stream';

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

const faulted = (thread: string, reason: string): RegistryRow => row({ thread, state: 'fault', fault: { since: minutesAgo(5), reason } });

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
  await waitForStream(sources);
  act(() => seedBoard(sources[0], docs));
  return sources;
};

const registry = (...rows: RegistryRow[]) => Object.fromEntries(rows.map((entry) => [entry.id, entry]));

describe('S1 the Registered strip', { tags: ['standard'] }, () => {
  it('is hidden until a thread is registered, and follows the registry stream', async () => {
    const sources = mount();
    await waitForStream(sources);
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
    expect(screen.getByText(/^Registered 4/)).toBeTruthy();
    expect(screen.getByText('1 waking')).toBeTruthy();
    expect(screen.getByText('1 fault')).toBeTruthy();
    const names = screen.getAllByText(/^(lead|beta|gamma|delta)$/).map((node) => node.textContent);
    expect(names).toEqual(['gamma', 'beta', 'lead', 'delta']);
    expect(screen.getByText('wake failed')).toBeTruthy();
    expect(screen.getByText('waking')).toBeTruthy();
    expect(screen.getByText('not listening')).toBeTruthy();
  });
});

describe('S1 the agents of the strip', { tags: ['standard'] }, () => {
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
});

describe('K1 the look of a chip', { tags: ['critical'] }, () => {
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
