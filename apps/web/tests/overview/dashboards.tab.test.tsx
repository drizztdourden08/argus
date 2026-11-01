/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { wholeText } from '../helpers/whole-text';
import { open } from './helpers/dashboards-world';
import { minutesAgo } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';

installHarness();

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
    expect(await screen.findByTitle('2 agents working')).toBeTruthy();
    expect(screen.getByTitle('5 Kanban items outstanding')).toBeTruthy();
    expect(screen.getByTitle('3 tests not passing')).toBeTruthy();
    expect(screen.getByText('1 decision')).toBeTruthy();
    expect(screen.getByText('1 failing')).toBeTruthy();
    expect(screen.getByText('1 blocked')).toBeTruthy();
    expect(screen.getByText('running')).toBeTruthy();
  });

  it('counts the worktrees against the running agents on each card and warns past two beyond', async () => {
    stubApi();
    await open({ meta: { worktrees: { counts: { rotp: { listed: 9, running: 2 }, beta: { listed: 3, running: 2 } } } } });
    const over = await screen.findByText('worktrees 9 / running agents 2');
    expect(over.closest('[title]')?.getAttribute('title')).toContain('some are not cleaned');
    const calm = screen.getByText('worktrees 3 / running agents 2');
    expect(calm.closest('[title]')?.getAttribute('title')).not.toContain('some are not cleaned');
  });

  it('puts the hot card first and gives a link to a board only to a card that has one', async () => {
    stubApi();
    await open();
    const buttons = await screen.findAllByRole('button', { name: /^Open the .+ board$/ });
    expect(buttons.map((button) => button.getAttribute('aria-label'))).toEqual(['Open the RotP board', 'Open the Beta board', 'Open the Delta board']);
    expect(buttons.map((button) => (button as HTMLButtonElement).disabled)).toEqual([false, false, true]);
    expect(screen.getByText('not started')).toBeTruthy();
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
    expect(within(strip).getByText('Beta')).toBeTruthy();
    expect(within(strip).getByText('wake failed')).toBeTruthy();
  });

  it('writes how long a wake has been on its way as a live time', async () => {
    stubApi();
    const waking = { rotp: { project: 'rotp', thread: 'lead', session: 'lead', role: 'coordinator', state: 'waking', since: minutesAgo(3), pendingWake: { since: minutesAgo(1.5) } } };
    await open({ registry: waking });
    const strip = await screen.findByRole('status', { name: 'Registered leads' });
    expect(within(strip).getByText('1 waking')).toBeTruthy();
    expect((await within(strip).findAllByText(wholeText(/^waking 2m$/))).length).toBeGreaterThan(0);
  });

  it('links a chip to the agents of its board and says in the tip what failed', async () => {
    stubApi();
    await open({ registry });
    const chip = await screen.findByRole('link', { name: /Wake failed: not listening since 10m ago/ });
    expect(chip.getAttribute('href')).toBe('/p/beta/#agents');
  });
});
