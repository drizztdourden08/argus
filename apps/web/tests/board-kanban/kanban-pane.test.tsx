/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { EMPTY_MEDIA } from '../../src/screens/board/tabs/kanban/pane/media/media.constants';
import { fieldProblem } from '../../src/screens/board/tabs/kanban/pane/model/field-problem';
import { inTab } from './kanban-drawer-dom';
import { MIN, ago, cleanKanban, item, mountKanban, openItem, where } from './kanban-harness';
import { bug, labelled, tabNames, tabOf } from './kanban-pane-dom';

cleanKanban();

describe('I-72 P6 the tabs of the pane', { tags: ['standard'] }, () => {
  it('gives a bug the tabs of its registry entry, and nothing about children or cases', async () => {
    await mountKanban({ items: [bug()] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Links', 'Discussion', 'Tests', 'Media', 'Builds', 'Activity']));
    expect(tabOf(dialog, 'Overview').getAttribute('aria-selected')).toBe('true');
  });

  it('gives an epic its Children tab and no Tests tab', async () => {
    await mountKanban({ items: [item('I-1', { type: 'epic', stage: 'open' })] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Children', 'Links', 'Discussion', 'Media', 'Builds', 'Activity']));
  });

  it('gives a test plan its Cases and Run tabs', async () => {
    await mountKanban({ items: [item('I-1', { type: 'test-plan', stage: 'draft' })] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Cases', 'Run', 'Links', 'Discussion', 'Media', 'Builds', 'Activity']));
  });

  it('reads an item without a type as a task', async () => {
    await mountKanban({ items: [item('I-1')] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Links', 'Discussion', 'Tests', 'Media', 'Builds', 'Activity']));
  });

  it('counts waiting links, open questions with pending to-dos, and failing tests in the tab badges', async () => {
    const blocked = [{ kind: 'item', id: 'I-2', label: 'Two' }, { kind: 'ask', id: 'Q-1', label: 'Why' }];
    const question = { id: 'Q-1', kind: 'question', item: 'I-1', author: 'worker', status: 'open', createdAt: ago(10 * MIN) };
    const todo = { id: 'H-1', item: 'I-1', thread: 'worker', text: 'Run it', status: 'pending', rank: 1, rev: 2, createdAt: ago(MIN) };
    const tests = [
      { id: 'T-1', item: 'I-1', title: 'One', mode: 'manual', status: 'fail', ac: [] },
      { id: 'T-2', item: 'I-1', title: 'Two', mode: 'manual', status: 'pass', ac: [] },
    ];
    await mountKanban({ items: [bug({ blocked }), item('I-2')], entries: [question], actions: [todo], tests });
    const dialog = await openItem('I-1');
    expect(tabOf(dialog, 'Links').textContent).toMatch(/2/);
    expect(tabOf(dialog, 'Discussion').textContent).toMatch(/2/);
    expect(tabOf(dialog, 'Tests').textContent).toBe('Tests1');
  });

  it('opens the tab the hash names, and writes the tab it is given to the hash', async () => {
    await mountKanban({ items: [bug()] }, undefined, '#I-1/links');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(tabOf(dialog, 'Links').getAttribute('aria-selected')).toBe('true');
    expect(within(dialog).getByRole('tabpanel', { name: 'Links' })).toBeTruthy();
    fireEvent.click(tabOf(dialog, 'Discussion'));
    await waitFor(() => expect(where.hash).toBe('#I-1/discussion'));
    expect(tabOf(dialog, 'Discussion').getAttribute('aria-selected')).toBe('true');
  });

  it('falls back to the first tab when the hash names one the type does not have', async () => {
    await mountKanban({ items: [bug()] }, undefined, '#I-1/children');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(tabOf(dialog, 'Overview').getAttribute('aria-selected')).toBe('true');
  });

  it('remembers the last tab per type', async () => {
    await mountKanban({ items: [bug(), item('I-2', { type: 'bug', stage: 'reported', title: 'Other' })] });
    const first = inTab(await openItem('I-1'), 'Tests');
    expect(localStorage.getItem('coord.pane.tab.bug')).toBe('tests');
    fireEvent.keyDown(first, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Work item' })).toBeNull());
    const second = await openItem('I-2');
    expect(tabOf(second, 'Tests').getAttribute('aria-selected')).toBe('true');
  });
});

describe('I-72 P6 the tabs that follow the type', { tags: ['standard'] }, () => {
  const epic = item('I-1', { type: 'epic', stage: 'open', title: 'Release' });
  const story = item('I-2', { type: 'story', stage: 'draft', parent: 'I-1' });
  const finished = item('I-3', { type: 'task', status: 'done', stage: 'done', parent: 'I-1' });

  it('lists the children of an epic, counts the open ones, and opens a child', async () => {
    await mountKanban({ items: [epic, story, finished] });
    const dialog = inTab(await openItem('I-1'), 'Children');
    expect(tabOf(dialog, 'Children').textContent).toMatch(/1/);
    expect(within(dialog).getByText('Children · 2')).toBeTruthy();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Open I-2: Item I-2' }));
    await waitFor(() => expect(where.hash).toBe('#I-2'));
  });

  it('adds a child of an allowed type under the item', async () => {
    const { posted } = await mountKanban({ items: [epic] });
    const dialog = inTab(await openItem('I-1'), 'Children');
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Add a child' }), { target: { value: 'Fresh' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add child' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'create', target: { c: 'items' }, set: { title: 'Fresh', type: 'story', parent: 'I-1', thread: 'worker' } });
  });

  it('shows the parents of an item as crumbs that open them', async () => {
    await mountKanban({ items: [epic, story] });
    const dialog = await openItem('I-2');
    fireEvent.click(within(within(dialog).getByRole('navigation', { name: 'Parents' })).getByRole('button', { name: 'Open I-1: Release' }));
    await waitFor(() => expect(where.hash).toBe('#I-1'));
  });
});

describe('I-72 P6 the cases and the slots of a test plan', { tags: ['standard'] }, () => {
  it('lists the cases of a plan with their status, and removes one', async () => {
    const plan = item('I-1', { type: 'test-plan', stage: 'draft', fields: { cases: ['T-1', 'T-2'] } });
    const tests = [
      { id: 'T-1', item: 'I-9', title: 'Wrong password', mode: 'manual', status: 'pass', ac: [] },
      { id: 'T-2', item: 'I-9', title: 'Right password', mode: 'manual', status: 'fail', ac: [] },
    ];
    const { posted } = await mountKanban({ items: [plan], tests });
    const dialog = inTab(await openItem('I-1'), 'Cases');
    expect(within(dialog).getByText('Cases · 2')).toBeTruthy();
    expect(within(dialog).getByText('Wrong password')).toBeTruthy();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Remove T-1' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'edit', target: { c: 'items', id: 'I-1' }, set: { 'fields.cases': ['T-2'] } });
  });

  it('refuses a case that is not a test id, and one already on the plan', async () => {
    const plan = item('I-1', { type: 'test-plan', stage: 'draft', fields: { cases: ['T-1'] } });
    const { posted } = await mountKanban({ items: [plan] });
    const dialog = inTab(await openItem('I-1'), 'Cases');
    const box = within(dialog).getByRole('textbox', { name: 'Add a case (T-n)' });
    fireEvent.change(box, { target: { value: 'T-1' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add case' }));
    expect(within(dialog).getByText('T-1 is already a case.')).toBeTruthy();
    fireEvent.change(box, { target: { value: 'nope' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add case' }));
    expect(within(dialog).getByText(fieldProblem('cases', 'T-1, nope', {}) as string)).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('asks for cases on the Run tab of a plan with none, and shows the empty state on the Media tab', async () => {
    await mountKanban({ items: [item('I-1', { type: 'test-plan', stage: 'draft' })] });
    const dialog = inTab(await openItem('I-1'), 'Run');
    expect(within(dialog).getByText('Add cases to this plan, then start a run.')).toBeTruthy();
    inTab(dialog, 'Media');
    expect(within(dialog).getByText(EMPTY_MEDIA)).toBeTruthy();
  });
});
