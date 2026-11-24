/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanKanban, mountKanban, openSettings } from './kanban-harness';

cleanKanban();

const REFUSAL = 'bad-column: a column needs an id and a name';

type Posted = Awaited<ReturnType<typeof mountKanban>>['posted'];

const sentConfig = (posted: Posted) => posted.find((sent) => sent.body.kind === 'kanban-config');

describe('I-206 the Columns and rules section', { tags: ['critical'] }, () => {
  it('sends the whole config through the kanban-config op', async () => {
    const view = await mountKanban();
    const dialog = await openSettings('Kanban');
    expect(within(dialog).getByText('This board runs the default columns and rules.')).toBeTruthy();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add a column' }));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save the columns and rules' }));
    await waitFor(() => expect(sentConfig(view.posted)).toBeTruthy());
    const post = sentConfig(view.posted);
    const kanban = post?.body.kanban as { columns: { id: string }[]; rules: unknown[] };
    expect(kanban.columns.map((column) => column.id)).toEqual(['todo', 'doing', 'review', 'done', 'column']);
    expect(kanban.rules).toHaveLength(3);
  });

  it('sends null for the default', async () => {
    const view = await mountKanban();
    const dialog = await openSettings('Kanban');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Use the default columns and rules' }));
    await waitFor(() => expect(sentConfig(view.posted)).toBeTruthy());
    expect(sentConfig(view.posted)?.body.kanban).toBeNull();
  });

  it('shows a refusal with its key and sentence', async () => {
    await mountKanban({}, (body) => (body.kind === 'kanban-config' ? { id: 'entry', status: 'refused', outcome: REFUSAL, entry: null, seq: 1 } : undefined));
    const dialog = await openSettings('Kanban');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add a column' }));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save the columns and rules' }));
    expect(await screen.findByText(`Refused: ${REFUSAL}.`)).toBeTruthy();
  });
});
