/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, doneDoc, itemDoc, testDoc } from './fixtures';
import { choose, entries, openMenu } from './menu-dom';
import { mountTab } from './mount-tab';

cleanAfterEach();

const owner = () => ({
  builds: [buildDoc('bundle-1')],
  items: [itemDoc('I-1'), itemDoc('I-2')],
  tests: [testDoc('T-1', { rev: 4 }), doneDoc('T-2', 'fail', 'broken', { rev: 3 })],
});

describe('the row menu', () => {
  it('lists the actions of a test to run, without Run again', async () => {
    await mountTab(owner());
    await openMenu('T-1');
    for (const name of ['Add a note...', 'Open details', 'Edit', 'Move to item', 'Archive', 'Delete', 'Copy id']) expect(await screen.findByRole('menuitem', { name })).toBeTruthy();
    expect(screen.queryByRole('menuitem', { name: 'Run again' })).toBeNull();
  });

  it('puts a done test back in To run', async () => {
    const { calls } = await mountTab(owner());
    await openMenu('T-2');
    await choose('Run again');
    await waitFor(() => expect(entries(calls)).toHaveLength(1));
    expect(entries(calls)[0]?.body).toMatchObject({ kind: 'rerun', author: 'human', replyTo: 'T-2' });
  });

  it('moves a test to another item with the revision the owner saw', async () => {
    const { calls } = await mountTab(owner());
    await openMenu('T-1');
    await choose('Move to item');
    fireEvent.click(await screen.findByRole('menuitemradio', { name: 'I-2 Item I-2' }));
    await waitFor(() => expect(entries(calls)).toHaveLength(1));
    expect(entries(calls)[0]?.body).toMatchObject({ kind: 'move', target: { c: 'tests', id: 'T-1' }, to: { item: 'I-2' }, prev: { item: 'I-1' }, baseRev: 4 });
  });

  it('archives and deletes with an Undo that sends the inverse', async () => {
    const { calls } = await mountTab(owner());
    await openMenu('T-1');
    await choose('Archive');
    await waitFor(() => expect(entries(calls)).toHaveLength(1));
    expect(entries(calls)[0]?.body).toMatchObject({ kind: 'archive', target: { c: 'tests', id: 'T-1' } });
    fireEvent.click(await screen.findByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(entries(calls)).toHaveLength(2));
    expect(entries(calls)[1]?.body).toMatchObject({ kind: 'unarchive', target: { c: 'tests', id: 'T-1' } });
    await openMenu('T-2');
    await choose('Delete');
    await waitFor(() => expect(entries(calls)).toHaveLength(3));
    expect(entries(calls)[2]?.body).toMatchObject({ kind: 'delete', target: { c: 'tests', id: 'T-2' } });
  });

  it('opens the details of the test', async () => {
    await mountTab(owner());
    await openMenu('T-1');
    await choose('Open details');
    expect(await screen.findByRole('dialog', { name: 'Test details' })).toBeTruthy();
  });
});
