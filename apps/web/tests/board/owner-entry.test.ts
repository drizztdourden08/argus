/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { OWNER_TEXT_MAX, ownerEntry } from '../../src/data/posts';
import { contextRequestFields, noteFields, replyFields, replyProblem, textProblem } from '../../src/screens/board/owner-posts';

const WHEN = new Date('2026-10-26T08:00:00.000Z');

describe('K1 owner entry', { tags: ['critical'] }, () => {
  it('carries a time based id the server accepts, the human author and the defaults of the page', () => {
    const entry = ownerEntry({ kind: 'ack', replyTo: 'N-3' }, WHEN);
    expect(entry.id).toMatch(/^O-[a-z0-9]+-[a-z0-9]{4}$/);
    expect(entry).toMatchObject({
      author: 'human',
      status: 'new',
      createdAt: '2026-10-26T08:00:00.000Z',
      kind: 'ack',
      replyTo: 'N-3',
      item: null,
      thread: null,
    });
  });

  it('never repeats an id within one millisecond', () => {
    const ids = new Set(Array.from({ length: 200 }, () => ownerEntry({ kind: 'ack' }, WHEN).id));
    expect(ids.size).toBeGreaterThan(190);
  });
});

describe('K1 owner post fields', { tags: ['critical'] }, () => {
  const question = { id: 'Q-1', item: 'I-4', thread: 'lead' };

  it('a reply points at the entry and keeps its item and thread', () => {
    expect(replyFields(question, 'decision', 'Yes')).toEqual({ kind: 'decision', text: 'Yes', replyTo: 'Q-1', item: 'I-4', thread: 'lead' });
    expect(replyFields({ id: 'N-1' }, 'ack', '')).toEqual({ kind: 'ack', text: '', replyTo: 'N-1', item: null, thread: null });
  });

  it('asking for a context is a followup naming the question', () => {
    const fields = contextRequestFields(question);
    expect(fields).toMatchObject({ kind: 'followup', item: 'I-4', thread: 'lead' });
    expect(fields.text).toBe('Please add context: what is this about and why does it need me? (Q-1)');
  });

  it('a note on an item goes to the thread of the item', () => {
    const item = { id: 'I-9', title: 'x', status: 'doing', thread: 'worker' };
    expect(noteFields(item, 'followup', 'hello')).toEqual({ kind: 'followup', text: 'hello', item: 'I-9', thread: 'worker' });
  });
});

describe('K1 owner text', { tags: ['critical'] }, () => {
  it('a long note is taken whole and one over the net gets a plain sentence', () => {
    expect(textProblem('x'.repeat(20000))).toBeNull();
    expect(textProblem('x'.repeat(OWNER_TEXT_MAX))).toBeNull();
    expect(textProblem('x'.repeat(OWNER_TEXT_MAX + 1))).toBe(
      'This text is 50001 characters and one post takes 50000 at most: shorten it or post it in two parts.',
    );
  });

  it('an empty reply asks for the text', () => {
    expect(replyProblem('   ', 'followup')).toBe('Write the reply.');
    expect(replyProblem('', 'decision')).toBe('Write the decision.');
    expect(replyProblem('ok', 'decision')).toBeNull();
  });
});
