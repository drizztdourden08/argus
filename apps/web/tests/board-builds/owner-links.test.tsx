/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { launcherDoc } from './fixtures';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

const item = (id: string, extra: object = {}) => ({ id, title: `Title of ${id}`, status: 'doing', thread: 'lead', type: 'task', ...extra });

const board = () => ({
  launchers: [launcherDoc('master', { master: true }), launcherDoc('web')],
  items: [item('I-3'), item('I-9', { status: 'done' }), item('Q-1', { type: 'question' })],
});

const cardOf = async (name: string): Promise<HTMLElement> => {
  await waitFor(() => expect(document.body.querySelector(`[data-launcher="${name}"]`)).not.toBeNull());
  return document.body.querySelector<HTMLElement>(`[data-launcher="${name}"]`) as HTMLElement;
};

const openMenu = async (name: string): Promise<void> => {
  vi.stubGlobal('CSS', { supports: () => false });
  fireEvent.click(within(await cardOf(name)).getByRole('button', { name: /^More actions for/ }));
  await screen.findByRole('menuitem', { name: /^Copy id/ });
};

const choose = async (name: string, label: RegExp): Promise<HTMLElement> => {
  await openMenu(name);
  fireEvent.click(screen.getByRole('menuitem', { name: label }));
  return screen.findByRole('dialog');
};

describe('I-85 Link a build to an item from the card menu', { tags: ['standard'] }, () => {
  it('offers the open items a build may serve and posts launcher-link with the chosen one', async () => {
    const { calls } = await mountBuilds(board());
    const box = await choose('web', /^Link to an item/);
    const link = within(box).getByRole<HTMLButtonElement>('button', { name: 'Link' });
    expect(link.disabled).toBe(true);
    fireEvent.click(within(box).getByRole('combobox', { name: 'Item' }));
    expect(screen.queryByRole('option', { name: /^I-9/ })).toBeNull();
    expect(screen.queryByRole('option', { name: /^Q-1/ })).toBeNull();
    fireEvent.click(await screen.findByRole('option', { name: /^I-3/ }));
    fireEvent.click(link);
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/entries', method: 'POST' });
    expect(calls[0]?.body).toMatchObject({ kind: 'launcher-link', author: 'human', target: { c: 'launchers', id: 'web' }, set: { for: 'I-3' } });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });
});

describe('I-85 Retire a build from the card menu', { tags: ['standard'] }, () => {
  it('posts launcher-retire with the optional note after the confirm', async () => {
    const { calls } = await mountBuilds(board());
    const box = await choose('web', /^Retire/);
    fireEvent.change(within(box).getByRole('textbox', { name: 'Why retire it (optional)' }), { target: { value: ' obsolete ' } });
    fireEvent.click(within(box).getByRole('button', { name: 'Retire' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ kind: 'launcher-retire', author: 'human', target: { c: 'launchers', id: 'web' }, set: { note: 'obsolete' } });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('Cancel posts nothing, and the master build has no Retire entry', async () => {
    const { calls } = await mountBuilds(board());
    const box = await choose('web', /^Retire/);
    fireEvent.click(within(box).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(calls).toEqual([]);
    await openMenu('master');
    expect(screen.queryByRole('menuitem', { name: /^Retire/ })).toBeNull();
    expect(screen.queryByRole('menuitem', { name: /^Link to an item/ })).not.toBeNull();
  });
});
