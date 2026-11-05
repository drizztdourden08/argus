/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanKanban, mountKanban, openSettings } from './kanban-harness';

cleanKanban();

describe('I-105 the board icon setting', { tags: ['critical'] }, () => {
  it('posts a brand mark as an icon entry', async () => {
    const view = await mountKanban();
    const dialog = await openSettings();
    const marks = within(dialog).getByRole('group', { name: 'Icon brand marks' });
    fireEvent.click(within(marks).getByRole('button', { name: 'Use the Brock mark' }));
    await screen.findByText('The board icon is saved.');
    const post = view.posted.find((sent) => sent.body.kind === 'icon');
    expect(post?.body).toMatchObject({ kind: 'icon', icon: 'brand:brock', author: 'human' });
  });

  it('posts nothing for a file type it does not take', async () => {
    const view = await mountKanban();
    const dialog = await openSettings();
    fireEvent.click(within(dialog).getByRole('radio', { name: 'Upload' }));
    const input = dialog.querySelector('input[type="file"]');
    if (!(input instanceof HTMLInputElement)) throw new Error('no file input');
    fireEvent.change(input, { target: { files: [new File(['x'], 'a.gif', { type: 'image/gif' })] } });
    expect(view.posted.some((sent) => sent.body.kind === 'icon')).toBe(false);
  });

  it('posts null for the default', async () => {
    const view = await mountKanban();
    view.change('meta', 'board', { project: 'alpha', name: 'Alpha', icon: 'brand:brock' });
    const dialog = await openSettings();
    const [reset] = within(dialog).getAllByRole('button', { name: 'Use the default' });
    if (!reset) throw new Error('no reset button');
    fireEvent.click(reset);
    await screen.findByText('The board icon is saved.');
    expect(view.posted.find((sent) => sent.body.kind === 'icon')?.body).toMatchObject({ icon: null });
  });
});

describe('I-105 the More menu', { tags: ['critical'] }, () => {
  it('no longer offers the lead avatar entry', async () => {
    await mountKanban();
    fireEvent.click(screen.getByRole('button', { name: 'More' }));
    await screen.findByRole('menuitem', { name: 'Legend' });
    expect(screen.queryByRole('menuitem', { name: /Lead avatar/ })).toBeNull();
  });
});
