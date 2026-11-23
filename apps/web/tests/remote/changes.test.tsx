/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { ago, MIN, prDoc } from './helpers/fixtures';
import { mountChanges } from './helpers/mount-changes';

cleanAfterEach();

const cardOf = async (id: string) => within(await screen.findByRole('group', { name: id }));

describe('S1 the Changes tab list', { tags: ['standard'] }, () => {
  it('shows the status, the branches, the age and the replay state', async () => {
    await mountChanges([prDoc('PR-1')]);
    const card = await cardOf('PR-1');
    expect(card.getByText('Open')).toBeTruthy();
    expect(card.getByText('feat/login into main')).toBeTruthy();
    expect(card.getByRole('button', { name: /^Open PR-1/ })).toBeTruthy();
    expect(card.getByText('5m')).toBeTruthy();
    expect(card.getByText('waits for the go')).toBeTruthy();
  });

  it('follows a pull request live', async () => {
    const { change } = await mountChanges([prDoc('PR-1')]);
    await cardOf('PR-1');
    await change('PR-1', prDoc('PR-1', { title: 'A new title', status: 'merged' }));
    const card = await cardOf('PR-1');
    expect(card.getByText('Merged')).toBeTruthy();
    expect(card.getAllByText('A new title').length).toBeGreaterThan(0);
  });
});

describe('P4 the Changes tab header and view', { tags: ['standard'] }, () => {
  it('shows the counts as the only header stats', async () => {
    await mountChanges([prDoc('PR-1')]);
    const stats = within(screen.getByRole('group', { name: 'Changes counts' }));
    expect(stats.getByText('Open 1')).toBeTruthy();
    expect(stats.getByText('Failing 0')).toBeTruthy();
    expect(stats.getByText('Merged 0')).toBeTruthy();
    expect(stats.getByText('Closed 0')).toBeTruthy();
    expect(stats.getByText('Not on GitHub 1')).toBeTruthy();
    expect(screen.queryByText(/pull request/)).toBeNull();
  });

  it('times a merged pull request by its merge', async () => {
    await mountChanges([prDoc('PR-2', { status: 'merged', mergedAt: ago(2 * MIN) })]);
    const card = await cardOf('PR-2');
    expect(card.getByText('2m')).toBeTruthy();
  });

  it('drops the branch and the replay in the compact view', async () => {
    const key = 'argus.tab.changes.layout.rotp';
    localStorage.setItem(key, 'compact');
    try {
      await mountChanges([prDoc('PR-1')]);
      await cardOf('PR-1');
      expect(screen.queryByText('feat/login into main')).toBeNull();
      expect(screen.queryByText('waits for the go')).toBeNull();
    } finally {
      localStorage.removeItem(key);
    }
  });
});

describe('K4 the Changes tab title', { tags: ['standard'] }, () => {
  it('draws the title as text, never as markup', async () => {
    const { container } = await mountChanges([prDoc('PR-1', { title: '<img src=x onerror=alert(1)>' })]);
    expect(await screen.findByText('<img src=x onerror=alert(1)>')).toBeTruthy();
    expect(container.querySelector('img')).toBeNull();
  });
});
