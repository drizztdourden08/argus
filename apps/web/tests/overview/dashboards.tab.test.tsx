/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { minutesAgo } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';
import type { ApiCall } from './helpers/stub-api';

const lineLike = (pattern: RegExp) => (_: string, element: Element | null): boolean => element?.children.length === 1 && pattern.test(element.textContent);

const COUNTS = { threads: 2, working: 1, blocked: 1, needsYou: 2, itemsOpen: 5, tests: 10, testsPass: 7, testsFailing: 1, agents: 2 };

const MASTER = { name: 'master', title: 'Main checkout', cmd: 'game run', current: true, window: false, play: 'launcher-master', updatedAt: null, by: null, lastRun: null };

const cards = {
  rotp: {
    key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/', counts: COUNTS, updatedAt: minutesAgo(1),
    attention: ['Q-3 needs a decision: ship it?', 'T-4 failing: login'],
    coordinator: { session: 'lead', link: null, lastSeen: minutesAgo(1), activity: 'working', note: null },
    master: MASTER,
  },
  beta: {
    key: 'beta', name: 'Beta', state: 'running', root: null, url: '/p/beta/', counts: { ...COUNTS, needsYou: 0, testsFailing: 0, blocked: 0, agents: 0, working: 0, itemsOpen: 1, tests: 0, testsPass: 0 }, master: null, updatedAt: minutesAgo(1),
    coordinator: { session: 'lead', link: null, lastSeen: minutesAgo(1), activity: 'idle', note: null },
  },
  delta: { key: 'delta', name: 'Delta', state: 'not started', root: null, url: '/p/delta/' },
};

installHarness();

const open = async (extra: Record<string, object> = {}) => {
  const view = renderAt('/');
  await feed(view.sources, { cards, ...extra });
  return view;
};

const posts = (calls: ApiCall[]): ApiCall[] => calls.filter((call) => call.method === 'POST');

describe('the totals and the cards', () => {
  it('counts the running leads and what needs you across them', async () => {
    stubApi();
    await open();
    expect(await screen.findByText('1 running')).toBeTruthy();
    expect(screen.getAllByText('2 need you').length).toBeGreaterThanOrEqual(1);
  });

  it('shows the state, the counters and the flags of a card', async () => {
    stubApi();
    await open();
    expect(await screen.findByText('agents 2')).toBeTruthy();
    expect(screen.getByText('items 5')).toBeTruthy();
    expect(screen.getByText('tests 3')).toBeTruthy();
    expect(screen.getByText('1 decision')).toBeTruthy();
    expect(screen.getByText('1 failing')).toBeTruthy();
    expect(screen.getByText('1 blocked')).toBeTruthy();
    expect(screen.getByText('running')).toBeTruthy();
  });

  it('puts the hot card first and gives a link to a board only to a card that has one', async () => {
    stubApi();
    await open();
    const links = await screen.findAllByRole('link', { name: 'Open board' });
    expect(links.map((link) => link.getAttribute('href'))).toEqual(['/p/rotp/', '/p/beta/']);
    expect(screen.getByText('not started')).toBeTruthy();
  });

  it('says no master set on a card without one', async () => {
    stubApi();
    await open();
    expect(await screen.findByText('no master set')).toBeTruthy();
  });

  it('starts the master launcher in the project scope and says so', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { run: { id: 'R-9' }, already: false } } : undefined));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Play' }));
    expect(await screen.findByText('Started RotP master as R-9.')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/p/rotp/runs', method: 'POST', body: { name: 'launcher-master', visible: false } }]);
  });

  it('offers Stop while the master run lives, and stops it in its scope', async () => {
    const live = { id: 'R-9', scope: 'rotp', name: 'launcher-master', status: 'running', queuedAt: minutesAgo(2), startedAt: minutesAgo(1.5), lastLine: 'window open' };
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { run: { id: 'R-9', status: 'stopped' } } } : undefined));
    await open({ cards: { ...cards, rotp: { ...cards.rotp, master: { ...MASTER, lastRun: live } } } });
    expect(await screen.findByText(lineLike(/^running 1m: window open$/))).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Stop' }));
    expect(await screen.findByText('Stopped RotP master (R-9).')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/p/rotp/runs/R-9/stop', method: 'POST', body: {} }]);
  });

  it('shows a disabled Play when the launcher has no run definition', async () => {
    stubApi();
    await open({ cards: { ...cards, rotp: { ...cards.rotp, master: { ...MASTER, play: null } } } });
    const play = await screen.findByRole('button', { name: 'Play' });
    expect((play as HTMLButtonElement).disabled).toBe(true);
  });
});

describe('the registered strip', () => {
  const registry = {
    rotp: { project: 'rotp', thread: 'lead', session: 'lead', role: 'coordinator', state: 'listening', since: minutesAgo(3) },
    beta: { project: 'beta', thread: 'lead', session: 'lead', role: 'coordinator', state: 'fault', since: minutesAgo(10), fault: { since: minutesAgo(10), reason: 'not-listening' } },
  };

  it('is hidden when no lead is registered', async () => {
    stubApi();
    await open();
    await screen.findByText('1 running');
    expect(screen.queryByRole('status', { name: 'Registered leads' })).toBeNull();
  });

  it('counts the registered leads and the faults, with a chip per lead', async () => {
    stubApi();
    await open({ registry });
    const strip = await screen.findByRole('status', { name: 'Registered leads' });
    expect(within(strip).getByText(/^Registered 2/)).toBeTruthy();
    expect(within(strip).getByText('1 fault')).toBeTruthy();
    expect(within(strip).getByText('Beta, wake failed')).toBeTruthy();
  });

  it('writes how long a wake has been on its way as a live time', async () => {
    stubApi();
    const waking = { rotp: { project: 'rotp', thread: 'lead', session: 'lead', role: 'coordinator', state: 'waking', since: minutesAgo(3), pendingWake: { since: minutesAgo(1.5) } } };
    await open({ registry: waking });
    const strip = await screen.findByRole('status', { name: 'Registered leads' });
    expect(within(strip).getByText('1 waking')).toBeTruthy();
    expect((await within(strip).findAllByText(lineLike(/^RotP, waking 1m$/))).length).toBeGreaterThan(0);
  });

  it('links a chip to the agents of its board and says in the tip what failed', async () => {
    stubApi();
    await open({ registry });
    const chip = await screen.findByRole('link', { name: /Wake failed: not listening since 10m ago/ });
    expect(chip.getAttribute('href')).toBe('/p/beta/#agents');
  });
});

describe('the send down dialog', () => {
  it('sends to every lead: the button opens the dialog, the text enables Send down, the post goes to the portfolio route', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-1', status: 'delivered', outcome: null } } : undefined));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog', { name: 'Send down to all projects' });
    expect(within(dialog).getByRole('button', { name: 'Send down' }).hasAttribute('disabled')).toBe(true);
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'ship it' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('O-1 sent down to all projects.')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/portfolio/send', method: 'POST', body: { project: 'all', text: 'ship it', decision: true } }]);
  });

  it('sends a note to one project from its card', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-2', status: 'waiting for a board', outcome: null } } : undefined));
    await open();
    fireEvent.click(await screen.findByTitle('Send a decision or a note to RotP'));
    const dialog = await screen.findByRole('dialog', { name: 'Send down to RotP' });
    fireEvent.click(within(dialog).getByText('Note'));
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'fyi' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('O-2 is kept for RotP until its board exists.')).toBeTruthy();
    expect(posts(calls)[0]?.body).toEqual({ project: 'rotp', text: 'fyi', decision: false });
  });

  it('keeps the dialog and says so when the post is refused', async () => {
    stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-3', status: 'refused', outcome: 'too long' } } : undefined));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'x' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('Refused: too long.')).toBeTruthy();
    expect(screen.getByRole('dialog')).toBeTruthy();
  });

  it('refuses a text over 50000 characters with a plain sentence and posts nothing', async () => {
    const calls = stubApi();
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'x'.repeat(50001) } });
    expect(await within(dialog).findByText(/This text is 50001 characters and one post takes 50000 at most/)).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Send down' }).hasAttribute('disabled')).toBe(true);
    expect(posts(calls)).toEqual([]);
  });
});

describe('running now and sent down', () => {
  it('lists the live runs of every project', async () => {
    stubApi((call) => (call.path.startsWith('/api/runs')
      ? { body: { runs: [{ id: 'R-5', scope: 'rotp', name: 'hello', status: 'running', queuedAt: minutesAgo(1), startedAt: minutesAgo(1), lastLine: 'compiling' }] } }
      : undefined));
    await open();
    expect(await screen.findByText('hello R-5')).toBeTruthy();
    expect(screen.getByText('compiling')).toBeTruthy();
  });

  it('counts a queued run from the moment it was queued and heads the list with its column labels', async () => {
    stubApi((call) => (call.path.startsWith('/api/runs')
      ? { body: { runs: [{ id: 'R-6', scope: 'rotp', name: 'hello', status: 'queued', queuedAt: minutesAgo(5.5), startedAt: null, lastLine: null }] } }
      : undefined));
    await open();
    const list = await screen.findByRole('list', { name: 'Running now' });
    expect(await within(list).findByText('5m')).toBeTruthy();
    expect(list.textContent).toContain('Elapsed');
    expect(list.textContent).toContain('Project');
  });

  it('offers a switcher that lists every project', async () => {
    stubApi();
    await open();
    expect(await screen.findByRole('combobox', { name: 'Switch project' })).toBeTruthy();
  });

  it('counts what was sent down', async () => {
    stubApi();
    await open({ entries: { 'O-1': { id: 'O-1', kind: 'decision', author: 'human', project: 'rotp', text: 'go ahead', createdAt: minutesAgo(4), status: 'delivered' } } });
    expect(await screen.findByText('Sent down (1)')).toBeTruthy();
    await waitFor(() => expect(screen.queryByText('Sent down (0)')).toBeNull());
  });
});
