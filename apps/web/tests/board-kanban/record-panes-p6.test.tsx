/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { kindOf } from '../../src/data/registry';
import { PANE_TABS } from '../../src/shell/records/pane/pane.constants';
import { defDoc, launcherDoc, runDoc } from '../board-builds/fixtures';
import { worktree } from '../board-worktrees/fixtures';
import { prDoc } from '../remote/helpers/fixtures';
import { tabNames } from './kanban-pane-dom';
import { cleanKanban, item, mountKanban } from './kanban-harness';

cleanKanban();

const labelsOf = (id: string): string[] => (kindOf(id)?.tabs ?? []).filter((tab) => Object.hasOwn(PANE_TABS, tab)).map((tab) => PANE_TABS[tab]?.label ?? tab);

const names = (dialog: HTMLElement): string[] => tabNames(dialog).map((name) => name.replace(/\d+$/, ''));

const board = () => ({
  items: [item('I-1')],
  feeds: [
    { coll: 'launchers', key: 'name', docs: [launcherDoc('web', { for: 'I-1' })] },
    { coll: 'launches', key: 'name', docs: [defDoc('web')] },
    { coll: 'runs', key: 'id', docs: [runDoc('R-2', 'launcher-web', { status: 'failed', exitCode: 3 })] },
    { coll: 'worktrees', key: 'id', docs: [worktree('a', 'stale', { item: 'I-1' })] },
    { coll: 'prs', key: 'id', docs: [prDoc('PR-1')] },
  ],
});

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
