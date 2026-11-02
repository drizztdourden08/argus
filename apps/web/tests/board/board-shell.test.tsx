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
  it('counts the worktrees against the running agents and warns when there are more than two beyond', async () => {
    await open('/p/rotp/', { listed: 4, runningAgents: 2 });
    const calm = await screen.findByText('worktrees 4 / running agents 2');
    expect(calm.closest('[title]')?.getAttribute('title')).not.toContain('some are not cleaned');
    cleanup();
    await open('/p/rotp/', { listed: 5, runningAgents: 2 });
    const over = await screen.findByText('worktrees 5 / running agents 2');
    expect(over.closest('[title]')?.getAttribute('title')).toContain('some are not cleaned');
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
