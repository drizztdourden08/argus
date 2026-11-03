/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { prDoc } from './helpers/fixtures';
import { mountChanges } from './helpers/mount-changes';

cleanAfterEach();

const cardOf = async (id: string, title = `Title of ${id}`) => within(await screen.findByRole('group', { name: `Pull request ${id}: ${title}` }));

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
    const card = await cardOf('PR-1', 'A new title');
    expect(card.getByText('Merged')).toBeTruthy();
  });
});

describe('K4 the Changes tab title', { tags: ['standard'] }, () => {
  it('draws the title as text, never as markup', async () => {
    const { container } = await mountChanges([prDoc('PR-1', { title: '<img src=x onerror=alert(1)>' })]);
    expect(await screen.findByText('<img src=x onerror=alert(1)>')).toBeTruthy();
    expect(container.querySelector('img')).toBeNull();
  });
});
