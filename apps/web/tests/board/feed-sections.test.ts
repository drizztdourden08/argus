/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { FEED_KIND } from '../../src/screens/board/feed-model';
import { at, entry, ids, manual, step, work } from './feed-docs';

describe('I-214 the cards of the activity', { tags: ['critical'] }, () => {
  it('keeps only questions, decisions and messages, never a to-do, a test, a notice or a lead step', () => {
    const result = work({
      entries: [entry({ id: 'Q-1' }), entry({ id: 'Q-2', options: undefined }), entry({ id: 'N-1', kind: 'notice' }), entry({ id: 'M-1', kind: 'message' })],
      actions: [step({ id: 'H-1' }), step({ id: 'H-3', assignee: 'lead:rotp' })],
      tests: [manual({ id: 'T-1' })],
    });
    expect(ids(result.asks).sort()).toEqual(['M-1', 'Q-1', 'Q-2']);
  });

  it('maps every card type to one registry kind', () => {
    expect(FEED_KIND).toEqual({ decision: 'decision', question: 'question', action: 'todo', test: 'test-case', run: 'test-run', message: 'message' });
  });

  it('indexes every visible item, finished ones too, for the item chip of a card', () => {
    const item = (id: string, over: object = {}) => ({ id, title: `Title ${id}`, status: 'doing', thread: 'lead', ...over });
    const { index } = work({ items: [item('I-1', { type: 'bug' }), item('I-2', { status: 'done' }), item('I-3', { archivedAt: at(1) }), item('I-4', { deletedAt: at(1) })] });
    expect([...index.keys()]).toEqual(['I-1', 'I-2']);
    expect(index.get('I-1')?.type).toBe('bug');
  });
});
