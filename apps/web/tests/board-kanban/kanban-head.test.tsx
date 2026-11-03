/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HEAD_SIZE_PX } from '../../src/shell/records/head.constants';
import { cardOf, cleanKanban, item, moreItem, mountKanban } from './kanban-harness';

cleanKanban();

const PIXEL = 'data:image/png;base64,iVBORw0KGgo=';
const LEAD = 'Assigned to alpha lead (coordinator)';

const headOf = (id: string): HTMLElement => cardOf(id);

const board = { items: [item('I-1', { assignee: 'owner' }), item('I-2', { assignee: 'coordinator' }), item('I-3', { assignee: 'lead:tessera' }), item('I-4', { assignee: 'builder' })] };

describe('I-83 the head row of a card', { tags: ['critical'] }, () => {
  it('draws every element at the one size of the owner avatar', async () => {
    await mountKanban(board);
    const head = within(headOf('I-1'));
    expect(HEAD_SIZE_PX).toBe(16);
    expect(head.getByRole('img', { name: 'Assigned to you' })).toBeTruthy();
    expect(headOf('I-1').querySelector('[data-type-mark] svg')?.getAttribute('width')).toBe(String(HEAD_SIZE_PX));
    expect(head.getByText('I-1')).toBeTruthy();
    expect(head.getByRole('button', { name: 'More actions for I-1' })).toBeTruthy();
  });

  it('draws the type as an outline in its colour, with no tile behind it', async () => {
    await mountKanban(board);
    const mark = headOf('I-1').querySelector('[data-type-mark]');
    expect(mark?.querySelectorAll('svg')).toHaveLength(1);
    expect(mark?.querySelector('svg')?.getAttribute('style') ?? '').toMatch(/color:\s*var\(--c-tag-[a-z]+\)|color:\s*var\(--c-text-dim\)/);
  });

  it('shows the brand mark of a lead, and the initials of a board without a brand', async () => {
    await mountKanban(board);
    const brand = headOf('I-3').querySelector('[data-lead-mark="brand"]');
    expect(brand?.querySelector('svg')).toBeTruthy();
    expect(within(headOf('I-3')).queryByText('TE')).toBeNull();
    const lead = headOf('I-2');
    expect(lead.querySelector('[data-lead-mark]')).toBeNull();
    expect(within(lead).getByRole('img', { name: 'Assigned to alpha lead (coordinator)' }).textContent).toBe('AL');
  });

  it('keeps the initials of another thread', async () => {
    await mountKanban(board);
    expect(within(headOf('I-4')).getByRole('img', { name: 'Assigned to builder' }).textContent).toBe('BU');
  });

  it('reads the avatar the board chose, a brand or an uploaded image', async () => {
    const view = await mountKanban(board);
    view.change('meta', 'board', { project: 'alpha', name: 'Alpha', avatar: 'brand:rotp' });
    expect(within(headOf('I-2')).getByRole('img', { name: LEAD }).closest('svg')).toBeTruthy();
    view.change('meta', 'board', { project: 'alpha', name: 'Alpha', avatar: PIXEL });
    expect(within(headOf('I-2')).getByRole('img', { name: LEAD }).getAttribute('src')).toBe(PIXEL);
  });
});

describe('I-83 the lead avatar setting', { tags: ['critical'] }, () => {
  it('saves a brand mark as an owner post and reads it back', async () => {
    const view = await mountKanban(board);
    await moreItem(/Lead avatar/);
    const dialog = await screen.findByRole('dialog', { name: 'Lead avatar' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Use the Brock mark' }));
    await screen.findByText('The lead avatar is saved.');
    const post = view.posted.find((sent) => sent.body.kind === 'avatar');
    expect(post?.body).toMatchObject({ kind: 'avatar', avatar: 'brand:brock', author: 'human' });
    view.change('meta', 'board', { project: 'alpha', name: 'Alpha', avatar: 'brand:brock' });
    expect(within(headOf('I-2')).getByRole('img', { name: LEAD }).closest('svg')).toBeTruthy();
  });

  it('refuses an image over the cap before it is posted', async () => {
    const view = await mountKanban(board);
    await moreItem(/Lead avatar/);
    const dialog = await screen.findByRole('dialog', { name: 'Lead avatar' });
    const input = dialog.querySelector('input[type="file"]');
    if (!(input instanceof HTMLInputElement)) throw new Error('no file input');
    const big = new File([new Uint8Array(300 * 1024)], 'big.png', { type: 'image/png' });
    fireEvent.change(input, { target: { files: [big] } });
    expect((await within(dialog).findByRole('alert')).textContent).toMatch(/over 180 KB/);
    expect(view.posted.some((sent) => sent.body.kind === 'avatar')).toBe(false);
  });
});
