/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { FEED_KIND, sectionOf } from '../../src/screens/board/feed-model';
import { at, entry, ids, manual, step, work } from './feed-docs';

describe('I-85 P4 the sections of the activity', { tags: ['critical'] }, () => {
  it('keeps what waits on the owner first, a to-do for a lead in its own section, notices in the log', () => {
    const result = work({
      entries: [entry({ id: 'Q-1' }), entry({ id: 'N-1', kind: 'notice' }), entry({ id: 'M-1', kind: 'message' })],
      actions: [step({ id: 'H-1' }), step({ id: 'H-2', assignee: 'owner' }), step({ id: 'H-3', assignee: 'lead:rotp' }), step({ id: 'H-4', assignee: 'worker' })],
    });
    expect(ids(result.waiting).sort()).toEqual(['H-1', 'H-2', 'M-1', 'Q-1']);
    expect(ids(result.leads).sort()).toEqual(['H-3', 'H-4']);
    expect(ids(result.notices)).toEqual(['N-1']);
  });

  it('maps every card type to one registry kind and puts each card in exactly one section', () => {
    expect(FEED_KIND).toEqual({ decision: 'decision', question: 'question', action: 'todo', test: 'test-case', bundle: 'test-case', message: 'message', notice: 'notice' });
    const cards = work({ entries: [entry({ id: 'Q-1' }), entry({ id: 'N-1', kind: 'notice' })], actions: [step({ id: 'H-1', assignee: 'lead:a' })], tests: [manual({ id: 'T-1' })] });
    const sections = [...cards.waiting, ...cards.leads, ...cards.notices];
    expect(sections.map((card) => sectionOf(card)).sort()).toEqual(['leads', 'log', 'waiting', 'waiting']);
  });

  it('indexes every visible item, finished ones too, for the item chip of a card', () => {
    const item = (id: string, over: object = {}) => ({ id, title: `Title ${id}`, status: 'doing', thread: 'lead', ...over });
    const { index } = work({ items: [item('I-1', { type: 'bug' }), item('I-2', { status: 'done' }), item('I-3', { archivedAt: at(1) }), item('I-4', { deletedAt: at(1) })] });
    expect([...index.keys()]).toEqual(['I-1', 'I-2']);
    expect(index.get('I-1')?.type).toBe('bug');
  });
});
