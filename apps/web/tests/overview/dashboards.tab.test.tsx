/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { open } from './helpers/dashboards-world';
import { minutesAgo } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';

installHarness();

describe('S1 the cards', { tags: ['standard'] }, () => {
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

describe('S1 the registered strip', { tags: ['standard'] }, () => {
  const registry = {
    beta: { project: 'beta', thread: 'lead', session: 'lead', role: 'coordinator', state: 'fault', since: minutesAgo(10), fault: { since: minutesAgo(10), reason: 'not-listening' } },
  };

  it('links a chip to the agents of its board and says in the tip what failed', async () => {
    stubApi();
    await open({ registry });
    const chip = await screen.findByRole('link', { name: /Wake failed: not listening since/ });
    expect(chip.getAttribute('href')).toBe('/p/beta/#agents');
  });
});
