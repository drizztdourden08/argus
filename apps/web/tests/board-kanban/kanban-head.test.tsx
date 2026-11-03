/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { HEAD_SIZE_PX } from '../../src/screens/board/tabs/kanban/head.constants';
import { cardOf, cleanKanban, item, moreItem, mountKanban } from './kanban-harness';

cleanKanban();

const PIXEL = 'data:image/png;base64,iVBORw0KGgo=';

const headOf = (id: string): HTMLElement => {
  const head = cardOf(id).querySelector('.head-row');
  if (!(head instanceof HTMLElement)) throw new Error(`no head row for ${id}`);
  return head;
};

const board = { items: [item('I-1', { assignee: 'owner' }), item('I-2', { assignee: 'coordinator' }), item('I-3', { assignee: 'lead:tessera' }), item('I-4', { assignee: 'builder' })] };

describe('I-83 the head row of a card', { tags: ['critical'] }, () => {
  it('draws every element at the one size of the owner avatar', async () => {
    await mountKanban(board);
    const head = headOf('I-1');
    expect(HEAD_SIZE_PX).toBe(16);
    expect(head.querySelector('.avatar--xs.avatar--owner')).toBeTruthy();
    expect(head.querySelector('[data-type-mark] svg')?.getAttribute('width')).toBe(String(HEAD_SIZE_PX));
    expect(head.querySelector('.head-id')).toBeTruthy();
    expect(head.querySelector('.btn')).toBeTruthy();
    const sheet = readFileSync(resolve(import.meta.dirname, '../../src/screens/board/tabs/kanban/sub-components/HeadRow.css'), 'utf8');
    expect(sheet.match(/var\(--size-16\)/g)?.length).toBeGreaterThanOrEqual(6);
  });

  it('draws the type as an outline in its colour, with no tile behind it', async () => {
    await mountKanban(board);
    const head = headOf('I-1');
    expect(head.querySelector('.type-mark')).toBeNull();
    const mark = head.querySelector('[data-type-mark]');
    expect(mark?.className).toBe('head-mark');
    expect(mark?.querySelector('svg')?.getAttribute('style') ?? '').toMatch(/color:\s*var\(--c-tag-[a-z]+\)|color:\s*var\(--c-text-dim\)/);
  });

  it('shows the brand mark of a lead as a square, and the initials of a board without a brand in a square', async () => {
    await mountKanban(board);
    const brand = headOf('I-3').querySelector('[data-lead-mark="brand"]');
    expect(brand?.querySelector('svg')).toBeTruthy();
    expect(headOf('I-3').querySelector('.avatar')).toBeNull();
    const lead = headOf('I-2');
    expect(lead.querySelector('[data-lead-mark]')).toBeNull();
    const square = lead.querySelector('.avatar.head-square');
    expect(square?.textContent).toBe('AL');
  });

  it('keeps the initials of another thread in a circle', async () => {
    await mountKanban(board);
    const circle = headOf('I-4').querySelector('.avatar');
    expect(circle?.textContent).toBe('BU');
    expect(circle?.classList.contains('head-square')).toBe(false);
  });

  it('reads the avatar the board chose, a brand or an uploaded image', async () => {
    const view = await mountKanban(board);
    view.change('meta', 'board', { project: 'alpha', name: 'Alpha', avatar: 'brand:rotp' });
    expect(headOf('I-2').querySelector('[data-lead-mark="brand"] svg')).toBeTruthy();
    view.change('meta', 'board', { project: 'alpha', name: 'Alpha', avatar: PIXEL });
    expect(headOf('I-2').querySelector('[data-lead-mark="image"] img')?.getAttribute('src')).toBe(PIXEL);
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
    expect(headOf('I-2').querySelector('[data-lead-mark="brand"]')).toBeTruthy();
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
