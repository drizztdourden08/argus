/* @layer renderer-app @kind test */
import { act, cleanup, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { snapshot } from '../helpers/snapshot';

afterEach(cleanup);

const cards = {
  rotp: { key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/' },
  argus: { key: 'argus', name: 'Argus', state: 'running', root: null, url: '/p/argus/' },
};

describe('routing', () => {
  it('shows the overview at / with the projects from the cards stream', async () => {
    const { sources } = renderAt('/');
    await waitFor(() => expect(sources).toHaveLength(1));
    const wanted = sources[0]?.url.split('?c=')[1]?.split(',') ?? [];
    expect(sources[0]?.url.startsWith('/api/p/_portfolio/stream?c=')).toBe(true);
    expect(wanted).toContain('cards');
    act(() => sources[0]?.emit('snapshot', snapshot('_portfolio', 'cards', cards)));
    expect(await screen.findByText('RotP')).toBeTruthy();
    const buttons = screen.getAllByRole('button', { name: /^Open the .+ board$/ });
    expect(buttons).toHaveLength(2);
  });

  it('shows the board page at /p/:key/ with the registered strip', async () => {
    const { sources } = renderAt('/p/rotp/');
    await waitFor(() => expect(sources).toHaveLength(1));
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
    expect(within(screen.getByRole('status', { name: 'Registered threads' })).getByText('lead')).toBeTruthy();
    for (const label of ['Agents', 'Activity', 'Kanban', 'Tests', 'Builds', 'Worktrees']) {
      expect(screen.getByRole('tab', { name: label })).toBeTruthy();
    }
  });

  it('opens the tab named by the hash', async () => {
    renderAt('/p/rotp/#tests');
    const tab = await screen.findByRole('tab', { name: 'Tests' });
    expect(tab.getAttribute('aria-selected')).toBe('true');
  });

  it('shows the not found page for a key that is not a project key', () => {
    renderAt('/p/Not_A_Key/');
    expect(screen.getByText('There is no such page.')).toBeTruthy();
  });

  it('shows the not found page for an unknown path', () => {
    renderAt('/nowhere');
    expect(screen.getByText('There is no such page.')).toBeTruthy();
  });
});
