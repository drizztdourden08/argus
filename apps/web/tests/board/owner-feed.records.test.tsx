/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { NOW, cleanEntryBoard, entryWork, mountEntryBoard } from '../helpers/entry-board';
import { byId, decision, freeText, manualStep, notice } from '../helpers/board-docs';
import { openBoard, resetBoard } from '../helpers/open-board';
import { mockPosts } from '../helpers/post-mock';

afterEach(resetBoard);

const cardOf = (id: string): HTMLElement => {
  const found = document.querySelector<HTMLElement>(`[data-record="${id}"]`);
  if (!found) throw new Error(`no card ${id}`);
  return found;
};

const before = (first: Element, second: Element): boolean => Boolean(first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING);

const leadStep = { ...manualStep, id: 'H-2', text: 'Rebuild the cache', assignee: 'lead:rotp' };

describe('I-85 P4 every card says what it is', { tags: ['standard'] }, () => {
  it('shows the kind icon, the code and the avatar of who must act on one row', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision, freeText), actions: byId(manualStep) });
    const asking = within(cardOf('Q-1'));
    expect(cardOf('Q-1').querySelector('[data-type-mark="decision"]')).not.toBeNull();
    expect(asking.getByText('Q-1')).toBeTruthy();
    expect(asking.getByRole('img', { name: 'Assigned to you' })).toBeTruthy();
    expect(asking.getByRole('button', { name: 'More actions for Q-1' })).toBeTruthy();
    expect(cardOf('Q-2').querySelector('[data-type-mark="question"]')).not.toBeNull();
    expect(cardOf('H-1').querySelector('[data-type-mark="todo"]')).not.toBeNull();
  });

  it('puts the question first, then the item as a chip with its own type icon, and no kind word', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision), items: { 'I-4': { id: 'I-4', title: 'Private repo', type: 'bug', thread: 'coordinator', status: 'doing', rank: 1 } } });
    const card = cardOf('Q-1');
    const chip = card.querySelector('[data-item-chip="I-4"]');
    expect(chip?.querySelector('[data-type-mark="bug"]')).not.toBeNull();
    expect(within(chip as HTMLElement).getByText('B-4 Private repo')).toBeTruthy();
    expect(before(within(card).getByRole('button', { name: /^Open Q-1/ }), chip as HTMLElement)).toBe(true);
    expect(within(card).queryByText('decision needed')).toBeNull();
  });
});

describe('I-85 P4 the answers', { tags: ['standard'] }, () => {
  it('shows the quick answers and folds the custom answer, which opens on click', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision) });
    const card = within(cardOf('Q-1'));
    expect(card.getByRole('button', { name: 'Yes, private recommended' })).toBeTruthy();
    expect(card.getByRole('button', { name: 'Not yet, stay local' })).toBeTruthy();
    const fold = card.getByText('Write an answer').closest('details');
    expect(fold?.open).toBe(false);
    fireEvent.click(card.getByText('Write an answer'));
    expect(fold?.open).toBe(true);
  });

  it('leaves the answer box open for a question that has no quick answer', async () => {
    mockPosts();
    await openBoard({ entries: byId(freeText) });
    expect(within(cardOf('Q-2')).getByRole('textbox', { name: 'Answer coordinator.' })).toBeTruthy();
    expect(within(cardOf('Q-2')).queryByText('Write an answer')).toBeNull();
  });

  it('clamps a long context to two lines and offers More, and a short one has none', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision, freeText) });
    expect(within(cardOf('Q-1')).getByRole('button', { name: 'More' })).toBeTruthy();
    expect(within(cardOf('Q-2')).queryByRole('button', { name: 'More' })).toBeNull();
  });
});

describe('I-85 P4 the sections of the feed', { tags: ['standard'] }, () => {
  it('orders waiting on you, then for the leads, then the folded log', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision, notice), actions: byId(manualStep, leadStep) });
    const waiting = screen.getByText('Waiting on you · 2');
    const leads = screen.getByText('For the leads · 1');
    const log = screen.getByText('Log · 1');
    expect(before(waiting, leads)).toBe(true);
    expect(before(leads, log)).toBe(true);
    expect(within(cardOf('H-2')).getByText('Rebuild the cache')).toBeTruthy();
    expect(log.closest('details')?.open).toBe(false);
  });

  it('keeps the to-do for the leads out of the owner count', async () => {
    mockPosts();
    await openBoard({ actions: byId(leadStep) });
    expect(screen.queryByText(/Waiting on you/)).toBeNull();
    expect(screen.getByText('For the leads · 1')).toBeTruthy();
  });
});

describe('I-85 P4 a card opens its pane', { tags: ['standard'] }, () => {
  cleanEntryBoard();

  it('opens the entry pane over the activity tab from the title', async () => {
    const question = { id: 'Q-1', kind: 'question', item: 'I-1', thread: 'worker', author: 'worker', status: 'open', text: 'Ship it today?', options: ['Ship now', 'Wait'], createdAt: NOW };
    await mountEntryBoard({ at: '/p/rotp/#activity', work: entryWork(), entries: { 'Q-1': question } });
    fireEvent.click(await screen.findByRole('button', { name: 'Open Q-1: Ship it today?' }));
    expect(await screen.findByRole('dialog', { name: 'Entry detail' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: /^Activity/ }).getAttribute('aria-selected')).toBe('true');
  });
});
