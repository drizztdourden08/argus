/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import type { Responder } from '../board-tests/mount-tab';
import { item } from '../board-kanban/kanban-harness';
import { defDoc, launcherDoc, runDoc } from './fixtures';
import { mountPaneBuilds } from './mount-pane-builds';

cleanAfterEach();

const board = () => ({
  items: [item('I-1')],
  launchers: [launcherDoc('web', { for: 'I-1' })],
  launches: [defDoc('web')],
  runs: [runDoc('R-2', 'launcher-web', { status: 'failed', exitCode: 3 })],
});

const answers: Responder = () => ({ status: 200, body: { lines: [], next: 0, dropped: 0, attachments: [] } });

describe('I-78 the run log of the item pane', { tags: ['critical'] }, () => {
  it('leaves the Work item dialog alone on the Builds tab: no other drawer panel is open over it', async () => {
    await mountPaneBuilds(board(), '#I-1/builds', answers);
    const pane = await screen.findByRole('dialog', { name: 'Work item' });
    await within(pane).findByText('Builds · 1');
    expect(screen.getAllByRole('dialog')).toEqual([pane]);
    expect(within(pane).queryByRole('dialog', { name: 'Run log', hidden: true })).toBeNull();
    expect(screen.queryByRole('dialog', { name: 'Run log' })).toBeNull();
  });

  it('opens the Run log of a build as a page-level drawer, outside the pane', async () => {
    await mountPaneBuilds(board(), '#I-1/builds', answers);
    const pane = await screen.findByRole('dialog', { name: 'Work item' });
    fireEvent.click(await within(pane).findByRole('button', { name: 'Full log of R-2' }));
    const log = await screen.findByRole('dialog', { name: 'Run log' });
    expect(within(log).getByText('launcher-web R-2')).toBeTruthy();
    expect(pane.contains(log)).toBe(false);
  });
});
