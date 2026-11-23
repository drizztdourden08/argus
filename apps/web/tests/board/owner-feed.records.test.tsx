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
    await openBoard({ entries: byId(decision, freeText) });
    const asking = within(cardOf('Q-1'));
    expect(cardOf('Q-1').querySelector('[data-type-mark="decision"]')).not.toBeNull();
    expect(asking.getByText('Q-1')).toBeTruthy();
    expect(asking.getByRole('img', { name: 'Assigned to you' })).toBeTruthy();
    expect(asking.getByRole('button', { name: 'More actions for Q-1' })).toBeTruthy();
    expect(cardOf('Q-2').querySelector('[data-type-mark="question"]')).not.toBeNull();
  });

  it('puts the code and the title on the header row of a question, then the context and the answers', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision) });
    const card = cardOf('Q-1');
    const head = card.querySelector('.record-header') as HTMLElement;
    const body = head.nextElementSibling as HTMLElement;
    expect(within(head).getByText('Q-1')).toBeTruthy();
    const title = within(card).getByText('Create the private repo?');
    expect(before(head, title)).toBe(true);
    expect(before(head, body)).toBe(true);
    expect(before(title, within(card).getByRole('button', { name: 'More' }))).toBe(true);
    expect(before(within(card).getByRole('button', { name: 'More' }), within(card).getByRole('button', { name: 'Custom answer' }))).toBe(true);
  });

  it('does not open a question card from a click on its body', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision) });
    fireEvent.click(cardOf('Q-1'));
    expect(screen.queryByRole('dialog', { name: 'Entry detail' })).toBeNull();
  });
});

describe('I-85 P4 the answers', { tags: ['standard'] }, () => {
  it('shows the quick answers and hides the custom answer field until Custom answer is pressed', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision) });
    const card = within(cardOf('Q-1'));
    expect(card.getByRole('button', { name: 'Yes, private recommended' })).toBeTruthy();
    expect(card.getByRole('button', { name: 'Not yet, stay local' })).toBeTruthy();
    expect(card.queryByRole('textbox')).toBeNull();
    fireEvent.click(card.getByRole('button', { name: 'Custom answer' }));
    expect(card.getByRole('textbox', { name: 'Or write your own answer' })).toBeTruthy();
  });

  it('leaves the answer box open for a question that has no quick answer', async () => {
    mockPosts();
    await openBoard({ entries: byId(freeText) });
    expect(within(cardOf('Q-2')).getByRole('textbox', { name: 'Answer coordinator.' })).toBeTruthy();
    expect(within(cardOf('Q-2')).queryByRole('button', { name: 'Custom answer' })).toBeNull();
  });

  it('clamps a long context to two lines and offers More, and a short one has none', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision, freeText) });
    expect(within(cardOf('Q-1')).getByRole('button', { name: 'More' })).toBeTruthy();
    expect(within(cardOf('Q-2')).queryByRole('button', { name: 'More' })).toBeNull();
  });
});

describe('I-214 the activity holds only questions and decisions', { tags: ['standard'] }, () => {
  it('shows the questions and decisions, and no to-do, notice or section header', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision, notice), actions: byId(manualStep, leadStep) });
    expect(screen.getByRole('list', { name: 'Questions and decisions' })).toBeTruthy();
    expect(screen.queryByText(/Questions and decisions ·/)).toBeNull();
    expect(document.querySelector('[data-record="Q-1"]')).not.toBeNull();
    expect(document.querySelector('[data-record="H-1"]')).toBeNull();
    expect(document.querySelector('[data-record="H-2"]')).toBeNull();
    expect(screen.queryByText(/Waiting on you/)).toBeNull();
    expect(screen.queryByText(/For the leads/)).toBeNull();
    expect(screen.queryByText(/^Log/)).toBeNull();
  });
});

describe('I-85 P4 a card opens its pane', { tags: ['standard'] }, () => {
  cleanEntryBoard();

  it('opens the entry pane over the activity tab from the code', async () => {
    const question = { id: 'Q-1', kind: 'question', item: 'I-1', thread: 'worker', author: 'worker', status: 'open', text: 'Ship it today?', options: ['Ship now', 'Wait'], createdAt: NOW };
    await mountEntryBoard({ at: '/p/rotp/#activity', work: entryWork(), entries: { 'Q-1': question } });
    fireEvent.click(await screen.findByRole('link', { name: 'Q-1' }));
    const pane = await screen.findByRole('dialog', { name: 'Entry detail' });
    const pageTab = screen.getAllByRole('tab', { name: /^Activity/ }).find((tab) => !pane.contains(tab));
    expect(pageTab?.getAttribute('aria-selected')).toBe('true');
  });
});
