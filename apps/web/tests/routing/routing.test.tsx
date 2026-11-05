/* @layer renderer-app @kind test */
import { act, cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { snapshot } from '../helpers/snapshot';
import { waitForStream } from '../helpers/wait-for-stream';

afterEach(cleanup);

const cards = {
  rotp: { key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/' },
  argus: { key: 'argus', name: 'Argus', state: 'running', root: null, url: '/p/argus/' },
};
const boards = Object.fromEntries(Object.keys(cards).map((key) => [`BD-${key}`, { key }]));

describe('S2 routing', { tags: ['standard'] }, () => {
  it('shows the overview at / with the projects from the cards stream', async () => {
    const { sources } = renderAt('/');
    await waitForStream(sources);
    const wanted = sources[0]?.url.split('?c=')[1]?.split(',') ?? [];
    expect(sources[0]?.url.startsWith('/api/p/_portfolio/stream?c=')).toBe(true);
    expect(wanted).toContain('cards');
    expect(wanted).toContain('boards');
    act(() => {
      sources[0]?.emit('snapshot', snapshot('_portfolio', 'cards', cards));
      sources[0]?.emit('snapshot', snapshot('_portfolio', 'boards', boards));
    });
    expect(await screen.findByText('RotP')).toBeTruthy();
    const buttons = screen.getAllByRole('button', { name: /^Open the .+ board$/ });
    expect(buttons).toHaveLength(2);
  });

  it('shows the board page at /p/:key/ with the registered strip', async () => {
    const { sources } = renderAt('/p/rotp/');
    await waitForStream(sources);
    expect(sources[0]?.url.startsWith('/api/p/rotp/stream?c=')).toBe(true);
    expect(sources[0]?.url.split('c=')[1]?.split(',')).toEqual(
      expect.arrayContaining(['actions', 'agents', 'builds', 'entries', 'items', 'log', 'meta', 'registry', 'tests', 'threads']),
    );
    act(() => {
      sources[0]?.emit('snapshot', snapshot('rotp', 'meta', { board: { project: 'rotp', name: 'Relic of the Past' } }));
      sources[0]?.emit('snapshot', snapshot('rotp', 'registry', {
        lead: { id: 'lead', project: 'rotp', thread: 'lead', session: 'lead', role: 'coordinator', state: 'listening' },
      }));
    });
    expect(await screen.findByRole('heading', { name: 'Relic of the Past' })).toBeTruthy();
    fireEvent.click(await screen.findByRole('button', { name: '1 registered' }));
    expect(within(await screen.findByRole('dialog', { name: 'Registered threads' })).getByText('lead')).toBeTruthy();
    for (const label of ['Agents', 'Activity', 'Kanban', 'Tests', 'Builds', 'Worktrees', 'Changes', 'Releases']) {
      expect(screen.getByRole('tab', { name: new RegExp(`^${label}`) })).toBeTruthy();
    }
  });

  it('opens the tab named by the hash', async () => {
    renderAt('/p/rotp/#tests');
    const tab = await screen.findByRole('tab', { name: /^Tests/ });
    expect(tab.getAttribute('aria-selected')).toBe('true');
  });

  it('has no Media tab, because media is a tab of one record pane', async () => {
    renderAt('/p/rotp/');
    await screen.findByRole('tab', { name: /^Kanban/ });
    expect(screen.queryByRole('tab', { name: /^Media/ })).toBeNull();
  });

  it('sends an old #media address to the Kanban', async () => {
    renderAt('/p/rotp/#media');
    const tab = await screen.findByRole('tab', { name: /^Kanban/ });
    expect(tab.getAttribute('aria-selected')).toBe('true');
  });
});

describe('S2 routing of pages that do not exist', { tags: ['standard'] }, () => {
  it('shows the not found page for a key that is not a project key, as Node does', () => {
    renderAt('/p/Not_A_Key/');
    expect(screen.getByText('There is no board for "Not_A_Key".')).toBeTruthy();
  });

  it('shows the not found page for a board the portfolio cards do not know,', async () => {
    const { portfolio } = renderAt('/p/ghost/');
    await waitFor(() => expect(portfolio()).toBeTruthy());
    act(() => portfolio()?.emit('snapshot', snapshot('_portfolio', 'cards', { rotp: cards.rotp })));
    expect(await screen.findByText('There is no board for "ghost".')).toBeTruthy();
  });

  it('shows the not found page for an unknown path', () => {
    renderAt('/nowhere');
    expect(screen.getByText('There is no such page.')).toBeTruthy();
  });
});
