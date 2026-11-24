/* @layer renderer-app @kind test */
import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { syncTone } from '../../src/shell/SyncStamp/behavior/sync-tone';
import { SyncStamp } from '../../src/shell';
import { feed } from '../overview/helpers/feed';
import { installHarness } from '../overview/helpers/harness';
import { stubApi } from '../overview/helpers/stub-api';
import { renderAt } from '../helpers/render-at';
import { liveStream } from '../helpers/wait-for-stream';
import { seedBoard } from '../helpers/seed-board';
import { wholeText } from '../helpers/whole-text';

installHarness();

const HOUR = 3_600_000;
const NOW = 1_800_000_000_000;

const hoursAgo = (hours: number): string => new Date(NOW - hours * HOUR).toISOString();

const syncOf = (hours: number, note = 'pushed 2 PRs') => ({ at: hoursAgo(hours), by: 'lead', note });

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(NOW);
});

afterEach(() => {
  vi.useRealTimers();
});

const stamp = async (expected: RegExp): Promise<HTMLElement> => {
  const [first] = await screen.findAllByText(wholeText(expected));
  if (!first) throw new Error('no sync stamp');
  return first;
};

describe('K1 the tone of the last remote sync', { tags: ['critical'] }, () => {
  it('is muted while fresh, amber from 20 h and red past 24 h', () => {
    const at = (hours: number, extra = 0): string => new Date(NOW - hours * HOUR - extra).toISOString();
    expect(syncTone(at(3), NOW)).toBe('muted');
    expect(syncTone(at(19), NOW)).toBe('muted');
    expect(syncTone(at(20), NOW)).toBe('warning');
    expect(syncTone(at(23), NOW)).toBe('warning');
    expect(syncTone(at(24), NOW)).toBe('warning');
    expect(syncTone(at(24, 1000), NOW)).toBe('danger');
    expect(syncTone(at(72), NOW)).toBe('danger');
  });

  it('is muted before the first record and for a time that is not a date', () => {
    expect(syncTone(undefined, NOW)).toBe('muted');
    expect(syncTone(null, NOW)).toBe('muted');
    expect(syncTone('not a date', NOW)).toBe('muted');
  });
});

describe('S1 the last remote sync stamp', { tags: ['standard'] }, () => {
  it('shows the age and keeps the note and the setter in the tip', () => {
    render(<SyncStamp sync={syncOf(3)} />);
    const text = screen.getByText(wholeText(/sync 3h/));
    expect(text.getAttribute('data-tone')).toBe('muted');
    expect(text.getAttribute('title')).toBe('Recorded by lead: pushed 2 PRs');
  });

  it('drops the words and the ago in its compact form', () => {
    render(<SyncStamp compact sync={syncOf(3)} />);
    expect(screen.getByText(wholeText(/^sync 3h$/))).toBeTruthy();
  });
});

const answerRemote = (syncs: object) => (call: { path: string }) =>
  call.path === '/api/remote' ? { body: { mode: 'remote', replay: {}, syncs } } : undefined;

describe('S2 the board header', { tags: ['standard'] }, () => {
  beforeEach(() => vi.stubGlobal('CSS', { supports: () => false }));

  const open = async (syncs: object) => {
    stubApi(answerRemote(syncs));
    const { sources } = renderAt('/p/rotp/');
    const board = await liveStream(sources, 'rotp');
    act(() => seedBoard(board, { meta: { board: { project: 'rotp', name: 'RotP', updatedAt: new Date(NOW).toISOString() } } }));
  };

  it('shows when the project last synced with the remote', async () => {
    await open({ rotp: syncOf(3), other: syncOf(30) });
    expect((await stamp(/^sync 3h$/)).getAttribute('data-tone')).toBe('muted');
  });

  it('says never when the project has no record', async () => {
    await open({ other: syncOf(1) });
    expect((await stamp(/^sync never$/)).getAttribute('data-tone')).toBe('muted');
  });
});

describe('S2 the overview cards', { tags: ['standard'] }, () => {
  const COUNTS = { threads: 1, working: 0, blocked: 0, needsYou: 0, itemsOpen: 0 };
  const seen = new Date(NOW).toISOString();
  const lead = { session: 'lead', link: null, lastSeen: seen, activity: 'idle', note: null };
  const cards = {
    rotp: { key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/', counts: COUNTS, updatedAt: seen, lead },
    beta: { key: 'beta', name: 'Beta', state: 'running', root: null, url: '/p/beta/', counts: COUNTS, updatedAt: seen, lead },
    gamma: { key: 'gamma', name: 'Gamma', state: 'running', root: null, url: '/p/gamma/', counts: COUNTS, updatedAt: seen, lead },
    delta: { key: 'delta', name: 'Delta', state: 'not started', root: null, url: '/p/delta/' },
  };
  const boards = Object.fromEntries(Object.keys(cards).map((key) => [`BD-${key}`, { key }]));

  const open = async (syncs: object) => {
    stubApi(answerRemote(syncs));
    const view = renderAt('/');
    await feed(view.sources, { cards, boards });
  };

  it('show the age with its tone on every started project', async () => {
    await open({ rotp: syncOf(3), beta: syncOf(22), gamma: syncOf(50), delta: syncOf(1) });
    expect((await stamp(/^sync 3h$/)).getAttribute('data-tone')).toBe('muted');
    expect((await stamp(/^sync 22h$/)).getAttribute('data-tone')).toBe('warning');
    expect((await stamp(/^sync 2d$/)).getAttribute('data-tone')).toBe('danger');
  });

  it('say never for a project with no record and show nothing on a card with no board', async () => {
    await open({ rotp: syncOf(3) });
    await stamp(/^sync 3h$/);
    expect(screen.getAllByText(wholeText(/^sync never$/))).toHaveLength(2);
    expect(screen.getAllByText(wholeText(/^sync (3h|never)$/))).toHaveLength(3);
  });
});
