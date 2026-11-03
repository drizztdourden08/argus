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

describe('S1 the row menu', { tags: ['standard'] }, () => {
  it('moves a test to another item with the revision the owner saw', async () => {
    const { calls } = await mountTab(owner());
    await openMenu('T-1');
    await choose('Move to item');
    fireEvent.click(await screen.findByRole('menuitemradio', { name: 'K-2 Item number 2' }));
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

  it('opens the test case pane of the test with the open parameter, over the Tests tab', async () => {
    await mountTab(owner());
    await openMenu('T-1');
    await choose('Open');
    await waitFor(() => expect(screen.getByTestId('search').textContent).toBe('?open=T-1'));
    expect(screen.getByTestId('hash').textContent).toBe('');
  });
});
