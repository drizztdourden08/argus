/* @layer renderer-app @kind test */
import { act, cleanup, screen, waitFor } from '@testing-library/react';
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
    const links = screen.getAllByRole('link', { name: 'Open board' });
    expect(links.map((link) => link.getAttribute('href'))).toEqual(['/p/argus/', '/p/rotp/']);
  });

  it('shows the board page at /p/:key/ with the thread list', async () => {
    const { sources } = renderAt('/p/rotp/');
    await waitFor(() => expect(sources).toHaveLength(1));
    expect(sources[0]?.url).toBe('/api/p/rotp/stream?c=meta,threads');
    act(() => {
      sources[0]?.emit('snapshot', snapshot('rotp', 'meta', { board: { project: 'rotp', name: 'Relic of the Past' } }));
      sources[0]?.emit('snapshot', snapshot('rotp', 'threads', { lead: { id: 'lead', name: 'lead', role: 'coordinator', status: 'active' } }));
    });
    expect(await screen.findByRole('heading', { name: 'Relic of the Past' })).toBeTruthy();
    expect(screen.getByText('lead')).toBeTruthy();
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
