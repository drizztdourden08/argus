/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { opened } from './kanban-drawer-dom';
import { cleanKanban, item, mountKanban, openItem, where } from './kanban-harness';

cleanKanban();

const three = { items: [item('I-1', { rank: 1 }), item('I-2', { rank: 2 }), item('I-3', { rank: 3 })] };

describe('the focus mode of the drawer', () => {
  it('toggles Expand and Collapse and remembers the choice', async () => {
    await opened();
    fireEvent.click(screen.getByRole('button', { name: 'Expand' }));
    expect(localStorage.getItem('coord.drawer.wide')).toBe('1');
    expect(screen.getByRole('button', { name: 'Collapse' }).getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByRole('heading', { name: /Login/ })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Collapse' }));
    expect(localStorage.getItem('coord.drawer.wide')).toBe('0');
    expect(screen.getByRole('button', { name: 'Expand' })).toBeTruthy();
  });

  it('opens expanded when the choice was saved', async () => {
    localStorage.setItem('coord.drawer.wide', '1');
    await mountKanban({ items: [item('I-1')] });
    fireEvent.click(await screen.findByRole('button', { name: /^Open I-1:/ }));
    expect(await screen.findByRole('button', { name: 'Collapse' })).toBeTruthy();
  });
});

describe('the link of the drawer', () => {
  it('copies the page address with the item anchor', async () => {
    const writeText = vi.fn(() => Promise.resolve());
    vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } });
    const { dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Copy a link to I-1' }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith(expect.stringMatching(/^https?:\/\/[^#]*#I-1$/)));
  });

  it('opens the drawer of the item named in the address, and a to-do opens its item', async () => {
    await mountKanban({ items: [item('I-1', { title: 'Login' })] }, undefined, '#I-1');
    expect(await screen.findByRole('dialog', { name: 'Work item' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: /Login/ })).toBeTruthy();
  });

  it('resolves a to-do anchor to the item it belongs to', async () => {
    await mountKanban({ items: [item('I-1', { title: 'Login' })], actions: [{ id: 'H-1', item: 'I-1', thread: 'worker', text: 'Do it', status: 'pending' }] }, undefined, '#H-1');
    expect(await screen.findByRole('heading', { name: /Login/ })).toBeTruthy();
  });

  it('writes the anchor when a card opens and the tab anchor when the drawer closes', async () => {
    await mountKanban({ items: [item('I-1')] });
    await openItem('I-1');
    expect(where.hash).toBe('#I-1');
    fireEvent.keyDown(screen.getByRole('dialog', { name: 'Work item' }), { key: 'Escape' });
    await waitFor(() => expect(where.hash).toBe('#kanban'));
  });
});

describe('walking the list with [ and ]', () => {
  it('moves to the next and the previous item, and the pager buttons follow', async () => {
    await mountKanban(three);
    await openItem('I-2');
    const dialog = screen.getByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Previous in the list ([)' }).disabled).toBe(false);
    fireEvent.keyDown(document.body, { key: ']' });
    await waitFor(() => expect(where.hash).toBe('#I-3'));
    expect(within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Next in the list (])' }).disabled).toBe(true);
    fireEvent.keyDown(document.body, { key: ']' });
    expect(where.hash).toBe('#I-3');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Previous in the list ([)' }));
    await waitFor(() => expect(where.hash).toBe('#I-2'));
    fireEvent.keyDown(document.body, { key: '[' });
    await waitFor(() => expect(where.hash).toBe('#I-1'));
    expect(within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Previous in the list ([)' }).disabled).toBe(true);
  });

  it('keeps the brackets typed into a field', async () => {
    await mountKanban(three);
    await openItem('I-2');
    fireEvent.keyDown(screen.getByRole('textbox', { name: 'Add a criterion' }), { key: ']' });
    expect(where.hash).toBe('#I-2');
  });

  it('has nowhere to go for an item that is not in the list on screen', async () => {
    await mountKanban({ items: [item('I-1'), item('I-8', { status: 'done' })] }, undefined, '#I-8');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Previous in the list ([)' }).disabled).toBe(true);
    expect(within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Next in the list (])' }).disabled).toBe(true);
  });
});

describe('the description editor keys', () => {
  it('leaves the editor on Escape without posting, and stops the text at 400 characters', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit description' }));
    const box = within(dialog).getByRole<HTMLTextAreaElement>('textbox', { name: 'Description' });
    expect(box.maxLength).toBe(400);
    fireEvent.change(box, { target: { value: 'gone' } });
    fireEvent.keyDown(box, { key: 'Escape' });
    expect(within(dialog).queryByRole('textbox', { name: 'Description' })).toBeNull();
    expect(posted).toHaveLength(0);
  });

  it('shows the counter near the limit', async () => {
    const { dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit description' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Description' }), { target: { value: 'x'.repeat(390) } });
    expect(within(dialog).getByText('390 / 400')).toBeTruthy();
  });
});
