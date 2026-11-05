/* @layer renderer-app @kind test */
import { act, cleanup, fireEvent, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { seedBoard } from '../helpers/seed-board';
import { snapshot } from '../helpers/snapshot';
import { waitForStream } from '../helpers/wait-for-stream';

beforeEach(() => vi.stubGlobal('CSS', { supports: () => false }));
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const NOW = Date.now();
const at = (minutesAgo: number): string => new Date(NOW - minutesAgo * 60_000).toISOString();

const fault = { id: 'ui', project: 'rotp', thread: 'ui', session: 'ui', role: 'thread', state: 'fault', fault: { since: at(5), reason: 'no-listener' } };

const WAKE = { 'W-1': { thread: 'ui', reason: 'kanban-stale', text: 'I-4 sits in doing', status: 'passive', listeners: 0, at: at(30) } };

const open = async () => {
  const { sources } = renderAt('/p/rotp/');
  await waitForStream(sources);
  act(() => {
    sources[0]?.emit('snapshot', snapshot('rotp', 'wakes', WAKE));
    seedBoard(sources[0], { meta: { board: { project: 'rotp', name: 'RotP', updatedAt: at(0) } }, registry: { ui: fault } });
  });
};

describe('S1 the Alerts popover', { tags: ['standard'] }, () => {
  it('counts faults and notices in the danger tone and lists each under its own section', async () => {
    await open();
    const button = await screen.findByRole('button', { name: /^Alerts/ });
    expect(within(button).getByText('2')).toBeTruthy();
    fireEvent.click(button);
    const panel = await screen.findByRole('dialog', { name: 'Alerts' });
    expect(within(panel).getByText('Faults (1)')).toBeTruthy();
    expect(within(within(panel).getByRole('list', { name: 'Faults' })).getByText(/Wake failed: no listener/)).toBeTruthy();
    expect(within(panel).getByText('Notices (1)')).toBeTruthy();
    expect(within(panel).queryByText(/^Stalled/)).toBeNull();
  });

  it('has the Alerts button without a badge when nothing needs attention', async () => {
    const { sources } = renderAt('/p/rotp/');
    await waitForStream(sources);
    act(() => seedBoard(sources[0], { meta: { board: { project: 'rotp', name: 'RotP', updatedAt: at(0) } } }));
    const button = await screen.findByRole('button', { name: /^Alerts/ });
    fireEvent.click(button);
    const panel = await screen.findByRole('dialog', { name: 'Alerts' });
    expect(within(panel).getByText('Nothing needs attention.')).toBeTruthy();
  });
});
