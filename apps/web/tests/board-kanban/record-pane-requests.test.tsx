/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { kindOf } from '../../src/data/registry';
import { PANE_TABS } from '../../src/shell/records/pane/pane.constants';
import { defDoc, launcherDoc, runDoc } from '../board-builds/fixtures';
import { tabNames } from './kanban-pane-dom';
import { HOUR, ago, cleanKanban, item, mountKanban } from './kanban-harness';

cleanKanban();

const labelsOf = (id: string): string[] => (kindOf(id)?.tabs ?? []).filter((tab) => Object.hasOwn(PANE_TABS, tab)).map((tab) => PANE_TABS[tab]?.label ?? tab);

const names = (dialog: HTMLElement): string[] => tabNames(dialog).map((name) => name.replace(/\d+$/, ''));

const request = {
  id: 'R-59',
  key: 'alpha',
  by: 'lead',
  at: ago(2 * HOUR),
  mode: 'targets',
  status: 'partial',
  branch: false,
  reason: null,
  note: null,
  targets: [{ id: 'a', label: 'agent-a' }, { id: 'b', label: 'agent-b' }],
  results: [
    { id: 'a', label: 'agent-a', ok: true, text: 'removed agent-a' },
    { id: 'b', label: 'agent-b', ok: false, text: 'it has uncommitted work' },
  ],
  skipped: null,
  startedAt: ago(HOUR),
  finishedAt: ago(HOUR),
};

const reply = (_body: object, url: string) => (url.endsWith('/worktrees/requests') ? { requests: [request] } : undefined);

const board = () => ({
  items: [item('I-1')],
  feeds: [
    { coll: 'launchers', key: 'name', docs: [launcherDoc('web', { for: 'I-1' })] },
    { coll: 'launches', key: 'name', docs: [defDoc('web')] },
    { coll: 'runs', key: 'id', docs: [runDoc('R-1', 'launcher-web', { status: 'failed', exitCode: 3 })] },
  ],
});

describe('I-87 a removal request opens its own pane while a bare R stays a run', { tags: ['standard'] }, () => {
  it('opens RQ-59 as the request R-59 with the tabs of its kind and its facts', async () => {
    await mountKanban(board(), reply, '?open=RQ-59');
    const dialog = await screen.findByRole('dialog');
    expect(names(dialog)).toEqual(labelsOf('removal-request'));
    expect(await within(dialog).findByText(/it has uncommitted work/)).toBeTruthy();
    expect(within(dialog).getAllByText(/agent-a/).length).toBeGreaterThan(0);
    expect(within(dialog).getAllByText('lead').length).toBeGreaterThan(0);
    expect(within(dialog).getAllByText('partial').length).toBeGreaterThan(0);
  });

  it('lists the worktrees of the request on its Links tab', async () => {
    await mountKanban(board(), reply, '?open=RQ-59&pane=links');
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByText('Worktrees · 2')).toBeTruthy();
  });

  it('opens R-1 as a run, not as a request', async () => {
    await mountKanban(board(), reply, '?open=R-1&pane=log');
    const dialog = await screen.findByRole('dialog');
    expect(names(dialog)).toEqual(labelsOf('run'));
  });
});
