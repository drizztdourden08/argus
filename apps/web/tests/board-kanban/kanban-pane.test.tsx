/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { fieldProblem } from '../../src/screens/board/tabs/kanban/pane/model/field-problem';
import { inTab } from './kanban-drawer-dom';
import type { Doc } from './kanban-harness';
import { MIN, ago, cleanKanban, item, mountKanban, openItem, thread, where } from './kanban-harness';

cleanKanban();

const bug = (extra: Doc = {}): Doc =>
  item('I-1', {
    type: 'bug',
    stage: 'reported',
    title: 'Lobby',
    fields: { severity: 'S2', environment: 'bundle-16', repro: '- open the lobby\n- look', foundIn: 'T-57' },
    ...extra,
  });

const tabNames = (dialog: HTMLElement): string[] => within(dialog).getAllByRole('tab').map((tab) => tab.textContent ?? '');

const tabOf = (dialog: HTMLElement, name: string): HTMLElement => within(dialog).getByRole('tab', { name: new RegExp(`^${name}`) });

const labelled = (labels: string[]) => labels.map((label) => expect.stringMatching(new RegExp(`^${label}`)));

describe('I-72 P6 the tabs of the pane', { tags: ['standard'] }, () => {
  it('gives a bug the tabs of its registry entry, and nothing about children or cases', async () => {
    await mountKanban({ items: [bug()] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Links', 'Discussion', 'Tests', 'Media', 'Activity']));
    expect(tabOf(dialog, 'Overview').getAttribute('aria-selected')).toBe('true');
  });

  it('gives an epic its Children tab and no Tests tab', async () => {
    await mountKanban({ items: [item('I-1', { type: 'epic', stage: 'open' })] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Children', 'Links', 'Discussion', 'Media', 'Activity']));
  });

  it('gives a test plan its Cases and Run tabs', async () => {
    await mountKanban({ items: [item('I-1', { type: 'test-plan', stage: 'draft' })] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Cases', 'Run', 'Links', 'Discussion', 'Media', 'Activity']));
  });

  it('reads an item without a type as a task', async () => {
    await mountKanban({ items: [item('I-1')] });
    const dialog = await openItem('I-1');
    expect(tabNames(dialog)).toEqual(labelled(['Overview', 'Links', 'Discussion', 'Tests', 'Media', 'Activity']));
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
    expect(tabOf(dialog, 'Tests').textContent).toContain('1/2');
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
    fireEvent.change(box, { target: { value: 'bad id!' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add case' }));
    expect(within(dialog).getByText(fieldProblem('cases', 'T-1, bad id!', {}) as string)).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('leaves the Run tab and the Media tab as empty slots', async () => {
    await mountKanban({ items: [item('I-1', { type: 'test-plan', stage: 'draft' })] });
    const dialog = inTab(await openItem('I-1'), 'Run');
    expect(within(dialog).getByText('The runner of this plan is not here yet.')).toBeTruthy();
    inTab(dialog, 'Media');
    expect(within(dialog).getByText('The images of this item show here.')).toBeTruthy();
  });
});

describe('I-72 P6 the fields of the Overview', { tags: ['standard'] }, () => {
  const mounted = async (extra: Doc = {}) => {
    const board = await mountKanban({ items: [bug(extra)], threads: [thread('worker')] });
    return { ...board, dialog: await openItem('I-1') };
  };

  it('shows the groups of the type, with each field in the control of its content kind', async () => {
    const { dialog } = await mounted();
    const report = within(within(dialog).getByRole('region', { name: 'Report' }));
    expect(report.getByRole('combobox', { name: 'Severity' })).toBeTruthy();
    expect(report.getByRole('button', { name: 'Edit Environment: bundle-16' })).toBeTruthy();
    expect(report.getByRole('button', { name: 'Edit Found in: T-57' })).toBeTruthy();
    expect(report.getByRole('combobox', { name: 'Reporter' })).toBeTruthy();
    const steps = within(within(dialog).getByRole('region', { name: 'Steps' }));
    expect(steps.getAllByRole('listitem').map((entry) => entry.textContent)).toEqual(['open the lobby', 'look']);
    expect(steps.getByRole('button', { name: 'Edit repro' })).toBeTruthy();
    expect(within(dialog).getByRole('region', { name: 'More' })).toBeTruthy();
  });

  it('shows no field group for a task', async () => {
    await mountKanban({ items: [item('I-1')] });
    const dialog = await openItem('I-1');
    expect(within(dialog).queryByRole('region', { name: 'Report' })).toBeNull();
    expect(within(dialog).queryByRole('region', { name: 'Details' })).toBeNull();
  });

  it('saves an enum pick as an edit of that field with the old value as prev', async () => {
    const { posted, dialog } = await mounted();
    fireEvent.click(within(dialog).getByRole('combobox', { name: 'Severity' }));
    fireEvent.click(await screen.findByRole('option', { name: 'S1 blocker' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'edit', target: { c: 'items', id: 'I-1' }, set: { 'fields.severity': 'S1' }, prev: { 'fields.severity': 'S2' }, baseRev: 3 });
  });

  it('saves a line the rules accept', async () => {
    const { posted, dialog } = await mounted();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit Environment: bundle-16' }));
    const box = within(dialog).getByRole('textbox', { name: 'Edit Environment' });
    fireEvent.change(box, { target: { value: 'bundle-17' } });
    fireEvent.keyDown(box, { key: 'Enter' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ set: { 'fields.environment': 'bundle-17' }, prev: { 'fields.environment': 'bundle-16' } });
  });

  it('refuses a line over its limit and a reference that is no id, with the message of the rules and no post', async () => {
    const { posted, dialog } = await mounted();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit Environment: bundle-16' }));
    const long = 'x'.repeat(121);
    const box = within(dialog).getByRole('textbox', { name: 'Edit Environment' });
    fireEvent.change(box, { target: { value: long } });
    fireEvent.keyDown(box, { key: 'Enter' });
    expect(within(dialog).getByText(fieldProblem('environment', long, {}) as string)).toBeTruthy();
    fireEvent.keyDown(box, { key: 'Escape' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit Found in: T-57' }));
    const ref = within(dialog).getByRole('textbox', { name: 'Edit Found in' });
    fireEvent.change(ref, { target: { value: 'bad id!' } });
    fireEvent.keyDown(ref, { key: 'Enter' });
    expect(within(dialog).getByText(fieldProblem('foundIn', 'bad id!', {}) as string)).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('refuses a lone paragraph in a rich field and saves bullets', async () => {
    const { posted, dialog } = await mounted();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit repro' }));
    const box = within(dialog).getByRole('textbox', { name: 'Repro' });
    fireEvent.change(box, { target: { value: 'One lone paragraph of words.' } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    expect(within(dialog).getByText(fieldProblem('repro', 'One lone paragraph of words.', {}) as string)).toBeTruthy();
    expect(posted).toHaveLength(0);
    fireEvent.change(box, { target: { value: '- first\n- second' } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ set: { 'fields.repro': '- first\n- second' }, prev: { 'fields.repro': '- open the lobby\n- look' } });
  });

  it('picks a person of the board for a person field', async () => {
    const { posted, dialog } = await mounted();
    fireEvent.click(within(dialog).getByRole('combobox', { name: 'Reporter' }));
    fireEvent.click(await screen.findByRole('option', { name: 'worker' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ set: { 'fields.reporter': 'worker' } });
  });
});

describe('I-72 P6 the facts of the Overview', { tags: ['standard'] }, () => {
  it('moves a typed item by stage', async () => {
    const { posted } = await mountKanban({ items: [bug()] });
    const dialog = await openItem('I-1');
    fireEvent.click(within(dialog).getByRole('combobox', { name: 'Stage' }));
    fireEvent.click(await screen.findByRole('option', { name: 'Confirmed' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'move', target: { c: 'items', id: 'I-1' }, to: { stage: 'confirmed' } });
  });

  it('keeps the status and the thread of an item without a type', async () => {
    await mountKanban({ items: [item('I-2')] });
    const dialog = await openItem('I-2');
    expect(within(dialog).getByRole('combobox', { name: 'Status' })).toBeTruthy();
    expect(within(dialog).getByRole('combobox', { name: 'Thread' })).toBeTruthy();
    expect(within(dialog).queryByRole('combobox', { name: 'Stage' })).toBeNull();
  });

  it('assigns the item to the owner', async () => {
    const { posted } = await mountKanban({ items: [bug()] });
    const dialog = await openItem('I-1');
    fireEvent.click(within(dialog).getByRole('combobox', { name: 'Assignee' }));
    fireEvent.click(await screen.findByRole('option', { name: 'You' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'edit', set: { assignee: 'owner' } });
  });

  it('neither shows nor edits the subject any more', async () => {
    await mountKanban({ items: [item('I-1', { subject: 'billing' })] });
    const dialog = await openItem('I-1');
    expect(within(dialog).queryByText('billing')).toBeNull();
    expect(within(dialog).queryByText('Subject')).toBeNull();
    expect(within(dialog).queryByRole('button', { name: /subject/i })).toBeNull();
  });
});
