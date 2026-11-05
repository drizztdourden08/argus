/* @layer renderer-app @kind test */
import { act, cleanup, fireEvent, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { seedBoard } from '../helpers/seed-board';
import { waitForStream } from '../helpers/wait-for-stream';

beforeEach(() => vi.stubGlobal('CSS', { supports: () => false }));
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const NOW = new Date().toISOString();

const lead = { name: 'coordinator', role: 'coordinator', status: 'active', activity: 'idle', session: 'RotP', lastSeen: NOW, link: 'chat://lead/1' };

const open = async (at = '/p/rotp/', worktrees: object | null = null) => {
  const { sources } = renderAt(at);
  await waitForStream(sources);
  const meta = { board: { project: 'rotp', name: 'RotP', updatedAt: NOW }, ...(worktrees ? { worktrees } : {}) };
  act(() => seedBoard(sources[0], { meta, threads: { coordinator: lead } }));
  return sources;
};

describe('S1 the board header', { tags: ['standard'] }, () => {
  it('links the worktrees and the running agents to their tabs and warns when there are more than two beyond', async () => {
    await open('/p/rotp/', { listed: 4, runningAgents: 2 });
    const calm = await screen.findByRole('link', { name: '4 worktrees' });
    expect(calm.getAttribute('href')).toBe('#worktrees');
    expect(calm.getAttribute('title')).not.toContain('some are not cleaned');
    expect(screen.getByRole('link', { name: '2 running' }).getAttribute('href')).toBe('#agents');
    cleanup();
    await open('/p/rotp/', { listed: 5, runningAgents: 2 });
    const over = await screen.findByRole('link', { name: '5 worktrees' });
    expect(over.getAttribute('title')).toContain('some are not cleaned');
  });
});

describe('S1 the More menu', { tags: ['standard'] }, () => {
  it('opens the legend in a dialog', async () => {
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'More' }));
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Legend' }));
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).getByText('ui not listening')).toBeTruthy();
  });
});
