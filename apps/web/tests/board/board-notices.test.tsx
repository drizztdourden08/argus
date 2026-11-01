/* @layer renderer-app @kind test */
import { act, cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { passiveNotices } from '../../src/screens/board/BoardHeader/behavior/passive-notices';
import { renderAt } from '../helpers/render-at';
import { seedBoard } from '../helpers/seed-board';
import { snapshot } from '../helpers/snapshot';

beforeEach(() => vi.stubGlobal('CSS', { supports: () => false }));
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const NOW = Date.now();
const at = (minutesAgo: number): string => new Date(NOW - minutesAgo * 60_000).toISOString();

const lead = { name: 'coordinator', role: 'coordinator', status: 'active', activity: 'idle', session: 'RotP', lastSeen: at(0) };

const WAKES = {
  'W-1': { thread: 'coordinator', reason: 'kanban-stale', text: 'I-4 sits in doing for 3 days', status: 'passive', listeners: 0, at: at(30) },
  'W-2': { thread: 'coordinator', reason: 'test-fail', text: 'T-1 failed', status: 'delivered', listeners: 1, at: at(20) },
  'W-3': { thread: 'ui', reason: 'worktree-review', text: '3 worktrees need a decision', status: 'passive', listeners: 0, at: at(10) },
};

const open = async (wakes: Record<string, object>) => {
  const { sources } = renderAt('/p/rotp/');
  await waitFor(() => expect(sources).toHaveLength(1));
  act(() => {
    sources[0]?.emit('snapshot', snapshot('rotp', 'wakes', wakes));
    seedBoard(sources[0], { meta: { board: { project: 'rotp', name: 'RotP', updatedAt: at(0) } }, threads: { coordinator: lead } });
  });
};

describe('the board notices', () => {
  it('keeps only the passive rows, newest first', () => {
    const docs = Object.entries(WAKES).map(([id, data]) => ({ id, ...data }));
    expect(passiveNotices(docs).map((doc) => doc.id)).toEqual(['W-3', 'W-1']);
  });

  it('counts the notices in the header and lists them on click', async () => {
    await open(WAKES);
    const button = await screen.findByRole('button', { name: /^notices/ });
    expect(within(button).getByText('2')).toBeTruthy();
    fireEvent.click(button);
    const panel = await screen.findByRole('dialog', { name: 'Board notices' });
    const list = within(panel).getByRole('list', { name: 'Notices' });
    const rows = within(list).getAllByRole('listitem');
    expect(rows).toHaveLength(2);
    expect(rows[0]?.textContent).toContain('worktree-review');
    expect(rows[0]?.textContent).toContain('3 worktrees need a decision');
    expect(rows[1]?.textContent).toContain('I-4 sits in doing for 3 days');
    expect(within(panel).queryByText('T-1 failed')).toBeNull();
  });

  it('shows nothing when there is no notice', async () => {
    await open({ 'W-2': WAKES['W-2'] });
    await screen.findByText('Coordination board');
    expect(screen.queryByRole('button', { name: /^notices/ })).toBeNull();
  });
});
