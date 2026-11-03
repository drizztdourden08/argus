/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { worktree } from '../board-worktrees/fixtures';
import { prDoc } from '../remote/helpers/fixtures';
import { cleanKanban, mountKanban } from './kanban-harness';
import { buildBoard, labelsOf, names } from './pane-kit';

cleanKanban();

const board = () =>
  buildBoard('R-2', [
    { coll: 'worktrees', key: 'id', docs: [{ ...worktree('a', 'stale', { item: 'I-1' }) }] },
    { coll: 'prs', key: 'id', docs: [prDoc('PR-1')] },
  ]);

describe('I-85 P6 one pane per kind of the Builds, Worktrees and Changes tabs', { tags: ['standard'] }, () => {
  it('opens a build on the Runs tab with its command and its runs', async () => {
    await mountKanban(board(), undefined, '?open=BL-web&pane=runs');
    const dialog = await screen.findByRole('dialog');
    expect(names(dialog)).toEqual(labelsOf('build'));
    expect(await within(dialog).findByText('Runs · 1')).toBeTruthy();
  });

  it('opens a run on its Log tab', async () => {
    await mountKanban(board(), undefined, '?open=R-2&pane=log');
    const dialog = await screen.findByRole('dialog');
    expect(names(dialog)).toEqual(labelsOf('run'));
  });

  it('opens a worktree with the tabs of its kind', async () => {
    await mountKanban(board(), undefined, '?open=WT-id-a');
    const dialog = await screen.findByRole('dialog');
    expect(names(dialog)).toEqual(labelsOf('worktree'));
  });

  it('opens a pull request on its Checks tab', async () => {
    await mountKanban(board(), undefined, '?open=PR-1&pane=checks');
    const dialog = await screen.findByRole('dialog');
    expect(names(dialog)).toEqual(labelsOf('change'));
  });
});
