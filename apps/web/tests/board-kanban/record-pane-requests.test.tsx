/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, ago, cleanKanban, mountKanban } from './kanban-harness';
import { buildBoard, labelsOf, names } from './pane-kit';

cleanKanban();

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

const board = () => buildBoard('R-1');

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
