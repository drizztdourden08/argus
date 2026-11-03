/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { fieldProblem } from '../../src/shell/records/pane/model/field-problem';
import type { Doc } from './kanban-harness';
import { cleanKanban, item, mountKanban, openItem, thread } from './kanban-harness';
import { bug } from './kanban-pane-dom';

cleanKanban();

const LONE = 'word '.repeat(30);

const mounted = async (extra: Doc = {}) => {
  const board = await mountKanban({ items: [bug(extra)], threads: [thread('worker')] });
  return { ...board, dialog: await openItem('I-1') };
};

describe('I-72 P6 the fields of the Overview', { tags: ['standard'] }, () => {
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
});

describe('I-72 P6 the rules of the fields of the Overview', { tags: ['standard'] }, () => {
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
    fireEvent.change(ref, { target: { value: 'bad id' } });
    fireEvent.keyDown(ref, { key: 'Enter' });
    expect(within(dialog).getByText(fieldProblem('foundIn', 'bad id', {}) as string)).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('refuses a lone paragraph in a rich field and saves bullets', async () => {
    const { posted, dialog } = await mounted();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit repro' }));
    const box = within(dialog).getByRole('textbox', { name: 'Repro' });
    fireEvent.change(box, { target: { value: LONE } });
    fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true });
    expect(within(dialog).getByText(fieldProblem('repro', LONE, {}) as string)).toBeTruthy();
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
