/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { installHarness } from '../overview/helpers/harness';
import { minutesAgo } from '../overview/helpers/fixtures';
import { stubApi } from '../overview/helpers/stub-api';
import type { ApiCall } from '../overview/helpers/stub-api';
import { COUNTS, item } from './helpers';
import { QUIET, renderCard } from './render-card';

installHarness();

const MASTER = { name: 'master', title: 'Main checkout', cmd: 'game run', current: true, window: false, play: 'launcher-master', updatedAt: null, by: null, lastRun: null };
const RUNNING_COUNTS = { ...COUNTS, threads: 2, working: 1, agents: 2, itemsOpen: 5 };
const rotp = (extra: object = {}) => item('rotp', { name: 'RotP', counts: RUNNING_COUNTS, master: MASTER, ...extra });
const posts = (calls: ApiCall[]): ApiCall[] => calls.filter((call) => call.method === 'POST');

describe('P4 the master of a board card', { tags: ['standard'] }, () => {
  it('starts the master launcher in the project scope and says so', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { run: { id: 'R-9' }, already: false } } : undefined));
    renderCard(rotp());
    fireEvent.click(await screen.findByRole('button', { name: 'Play' }));
    expect(await screen.findByText('Started RotP master as R-9.')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/p/rotp/runs', method: 'POST', body: { name: 'launcher-master', visible: false } }]);
  });

  it('offers Stop while the master run lives, and stops it in its scope', async () => {
    const live = { id: 'R-9', scope: 'rotp', name: 'launcher-master', status: 'running', queuedAt: minutesAgo(2), startedAt: minutesAgo(1.5), lastLine: 'window open' };
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { run: { id: 'R-9', status: 'stopped' } } } : undefined));
    renderCard(rotp({ master: { ...MASTER, lastRun: live } }));
    fireEvent.click(await screen.findByRole('button', { name: 'Stop' }));
    expect(await screen.findByText('Stopped RotP master (R-9).')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/p/rotp/runs/R-9/stop', method: 'POST', body: {} }]);
  });

  it('shows a disabled Play when the launcher has no run definition', async () => {
    stubApi();
    renderCard(rotp({ master: { ...MASTER, play: null } }));
    const play = await screen.findByRole('button', { name: 'Play' });
    expect((play as HTMLButtonElement).disabled).toBe(true);
  });
});

describe('P4 sending down from a board card', { tags: ['standard'] }, () => {
  it('sends to every lead: the button opens the dialog, the text enables Send down, the post goes to the portfolio route', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-1', status: 'delivered', outcome: null } } : undefined));
    renderCard(rotp());
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog', { name: 'Send down to all projects' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(posts(calls)).toEqual([]);
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'ship it' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('O-1 sent down to all projects.')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/portfolio/send', method: 'POST', body: { project: 'all', text: 'ship it', decision: true } }]);
  });

  it('sends a note to one lead from its card', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-2', status: 'waiting for a board', outcome: null } } : undefined));
    renderCard(rotp());
    fireEvent.click(await screen.findByTitle('Send a decision or a note to RotP'));
    const dialog = await screen.findByRole('dialog', { name: /^Send down to / });
    fireEvent.click(within(dialog).getByText('Note'));
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'fyi' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText(/^O-2 is kept for .+ until its board exists\.$/)).toBeTruthy();
    expect(posts(calls)[0]?.body).toEqual({ project: 'rotp', text: 'fyi', decision: false });
  });

  it('keeps the popover and says so when the post is refused', async () => {
    stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-3', status: 'refused', outcome: 'too long' } } : undefined));
    renderCard(rotp());
    fireEvent.click(await screen.findByTitle('Send a decision or a note to RotP'));
    const dialog = await screen.findByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'x' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('Refused: too long.')).toBeTruthy();
    expect(screen.getByRole('dialog')).toBeTruthy();
  });
});

describe('P4 stall and fault on a board card', { tags: ['standard'] }, () => {
  it('shows a stalled signal with its count, and none when nothing stalls', async () => {
    stubApi();
    renderCard(rotp(), { ...QUIET, stalls: 2 });
    expect(await screen.findByRole('button', { name: /stalled/ })).toBeTruthy();
  });

  it('has no stalled signal when nothing stalls', async () => {
    stubApi();
    renderCard(rotp());
    await screen.findByRole('button', { name: 'Play' });
    expect(screen.queryByRole('button', { name: /stalled/ })).toBeNull();
  });

  it('says wake failed for a wake fault and board closed for a closed board', async () => {
    stubApi();
    const view = renderCard(rotp(), { ...QUIET, fault: { since: minutesAgo(10), reason: 'not-listening' } });
    expect(await screen.findByRole('button', { name: /wake failed/ })).toBeTruthy();
    view.unmount();
    renderCard(rotp(), { ...QUIET, fault: { since: minutesAgo(10), reason: 'board-closed' } });
    expect(await screen.findByRole('button', { name: /board closed/ })).toBeTruthy();
  });
});

describe('P4 a board that has not started', { tags: ['standard'] }, () => {
  it('has no Play, no send, no link to a board, and a dashed look', async () => {
    stubApi();
    renderCard(item('delta', null, { name: 'Delta' }));
    expect(await screen.findByText('Delta')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Play' })).toBeNull();
    expect(screen.queryByTitle(/^Send a decision or a note to/)).toBeNull();
    expect(screen.queryByRole('button', { name: 'Open the Delta board' })).toBeNull();
    expect(document.querySelector('[data-swap-item]')?.getAttribute('data-started')).toBe('false');
  });

  it('keeps data-swap-item on the root and links a started card to its board', async () => {
    stubApi();
    renderCard(rotp());
    expect(await screen.findByRole('button', { name: 'Open the RotP board' })).toBeTruthy();
    expect(document.querySelector('[data-swap-item="BD-rotp"]')).toBeTruthy();
  });
});

describe('P4 the worktree counter of a board card', { tags: ['standard'] }, () => {
  it('shows the board worktree count and opens its Worktrees tab', async () => {
    stubApi();
    renderCard(rotp(), QUIET, { rotp: { listed: 3, running: 2 } });
    expect(await screen.findByText('worktrees 3 / running agents 2')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Open the worktrees of RotP' }));
    expect(screen.getByTestId('where').textContent).toContain('worktrees');
  });

  it('shows no counter for a board without worktree data', async () => {
    stubApi();
    renderCard(rotp());
    await screen.findByRole('button', { name: 'Play' });
    expect(screen.queryByText(/^worktrees /)).toBeNull();
  });
});
