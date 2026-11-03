/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { Entry } from '../../src/data/kanban';
import { askLinks } from '../../src/screens/board/tabs/kanban/model';
import { before, entryRef, itemRef } from './kanban-docs';

const decision = (id: string, extra: Partial<Entry> = {}) => entryRef(id, { kind: 'decision', status: 'pending', ...extra });

describe('I-68 the questions and decisions that wait on an item', { tags: ['critical'] }, () => {
  it('keeps an open question and a pending decision of the item', () => {
    const links = askLinks([itemRef('I-1')], [decision('D-1', { createdAt: before(1000) }), entryRef('Q-1', { createdAt: before(5000) })]);
    expect(links.get('I-1')).toEqual([
      { id: 'Q-1', kind: 'question', via: 'item' },
      { id: 'D-1', kind: 'decision', via: 'item' },
    ]);
  });

  it('drops a question that is answered, settled by the owner or deleted, and a decision already applied', () => {
    const links = askLinks(
      [itemRef('I-1')],
      [
        entryRef('Q-1', { status: 'answered' }),
        entryRef('Q-2'),
        entryRef('H-1', { kind: 'followup', author: 'human', replyTo: 'Q-2', item: null }),
        entryRef('Q-3', { deletedAt: before(1000) }),
        entryRef('Q-4', { archivedAt: before(1000) }),
        decision('D-1', { status: 'applied' }),
      ],
    );
    expect(links.get('I-1')).toBeUndefined();
  });

  it('links a blocker that names a live entry of another item, once', () => {
    const items = [itemRef('I-1', { blocker: 'Q-4' }), itemRef('I-2', { blocker: 'D-2' })];
    const links = askLinks(items, [entryRef('Q-4', { item: 'I-9' }), decision('D-2', { item: 'I-2' })]);
    expect(links.get('I-1')).toEqual([{ id: 'Q-4', kind: 'question', via: 'blocker' }]);
    expect(links.get('I-2')).toEqual([{ id: 'D-2', kind: 'decision', via: 'blocker' }]);
    expect(links.get('I-9')).toEqual([{ id: 'Q-4', kind: 'question', via: 'item' }]);
  });

  it('leaves an external blocker, an unknown id and a blocker on an answered question as text', () => {
    const items = [itemRef('I-1', { blocker: 'external: legal, the contract' }), itemRef('I-2', { blocker: 'D-11' }), itemRef('I-3', { blocker: 'Q-5' })];
    const links = askLinks(items, [entryRef('Q-5', { item: 'I-9', status: 'answered' })]);
    expect(links.get('I-1')).toBeUndefined();
    expect(links.get('I-2')).toBeUndefined();
    expect(links.get('I-3')).toBeUndefined();
  });
});
