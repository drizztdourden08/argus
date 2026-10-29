/* @layer renderer-app @kind test */
import { act, cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { seedBoard } from '../helpers/seed-board';

beforeEach(() => vi.stubGlobal('CSS', { supports: () => false }));
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const NOW = new Date().toISOString();

const lead = { name: 'coordinator', role: 'coordinator', status: 'active', activity: 'idle', session: 'RotP', lastSeen: NOW, link: 'chat://lead/1' };

const open = async (at = '/p/rotp/') => {
  const { sources } = renderAt(at);
  await waitFor(() => expect(sources).toHaveLength(1));
  act(() => seedBoard(sources[0], { meta: { board: { project: 'rotp', name: 'RotP', updatedAt: NOW } }, threads: { coordinator: lead } }));
  return sources;
};

describe('the board header', () => {
  it('names the page, shows the lead with its status and when the board changed', async () => {
    await open();
    expect(await screen.findByText('Coordination board')).toBeTruthy();
    expect(screen.getByRole('heading', { level: 1, name: 'RotP' })).toBeTruthy();
    expect(await screen.findByText('active')).toBeTruthy();
    expect(screen.getByRole('link', { name: 'Open the chat of the lead' }).getAttribute('href')).toBe('chat://lead/1');
    expect(screen.getByText(/board updated/)).toBeTruthy();
  });

  it('puts the tab bar right under the registered strip, with a count on every tab', async () => {
    await open();
    const tabs = await screen.findAllByRole('tab');
    expect(tabs.map((tab) => tab.textContent)).toEqual(['Agents0', 'Activity0', 'Kanban0', 'Tests0', 'Builds0', 'Worktrees0', 'Changes0', 'Releases0']);
    expect(screen.queryByText('Work items')).toBeNull();
  });

  it('holds the activity feed in the Activity tab', async () => {
    await open('/p/rotp/#activity');
    expect(await screen.findByRole('heading', { name: 'Activity' })).toBeTruthy();
    expect(screen.getByRole('radiogroup', { name: 'Activity view' })).toBeTruthy();
  });

  it('shows the Focus select under the tabs on every tab', async () => {
    await open('/p/rotp/#builds');
    expect(await screen.findByRole('combobox', { name: 'Focus' })).toBeTruthy();
  });
});

describe('the More menu', () => {
  it('lists trash, resources, legend, hidden threads and purge', async () => {
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'More' }));
    const names = (await screen.findAllByRole('menuitem')).map((item) => item.textContent);
    expect(names[0]).toBe('Trash (0)');
    expect(names[1]).toBe('Resources and locks (0 held)');
    expect(names[2]).toBe('Legend');
    expect(names[3]).toMatch(/^Show hidden \(0\)/);
    expect(names[4]).toBe('Purge threads and records...');
  });

  it('opens the legend in a dialog', async () => {
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'More' }));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Legend' }));
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByText('ui not listening')).toBeTruthy();
  });
});
