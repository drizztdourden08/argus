/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { askMode, contextParts, hasContext } from '../../src/screens/board/feed-model';
import { at, entry, human, ids, manual, step, work } from './feed-docs';

describe('K1 what asks the owner', { tags: ['critical'] }, () => {
  it('a question with options is a decision, one without is free text, an old decision wording is information', () => {
    expect(askMode(entry({ id: 'Q-1' }))).toBe('decide');
    expect(askMode(entry({ id: 'Q-2', options: undefined }))).toBe('answer');
    expect(askMode(entry({ id: 'Q-3', options: [], needs: 'decision' }))).toBe('info');
    expect(askMode(entry({ id: 'N-1', kind: 'notice' }))).toBe('info');
  });

  it('open questions split into decision and question cards, newest first', () => {
    const { waiting } = work({
      entries: [
        entry({ id: 'Q-1', createdAt: at(1) }),
        entry({ id: 'Q-2', options: undefined, createdAt: at(9) }),
      ],
    });
    expect(waiting.map((card) => [card.type, card.data.id])).toEqual([['question', 'Q-2'], ['decision', 'Q-1']]);
  });

  it('an answered or dismissed question leaves the feed at once, a closed one never shows', () => {
    const { waiting } = work({
      entries: [
        entry({ id: 'Q-1' }),
        entry({ id: 'Q-2' }),
        entry({ id: 'Q-3' }),
        entry({ id: 'Q-4', status: 'closed' }),
        human({ id: 'O-1', kind: 'decision', replyTo: 'Q-1' }),
        human({ id: 'O-2', kind: 'ack', replyTo: 'Q-2' }),
      ],
    });
    expect(ids(waiting)).toEqual(['Q-3']);
  });

  it('notices wait in their own group and a messages needs the owner', () => {
    const result = work({
      entries: [
        entry({ id: 'N-1', kind: 'notice', createdAt: at(3) }),
        entry({ id: 'M-1', kind: 'message', createdAt: at(4) }),
        entry({ id: 'N-2', kind: 'notice', createdAt: at(5) }),
        human({ id: 'O-1', kind: 'ack', replyTo: 'N-2' }),
      ],
    });
    expect(result.notices.map((card) => card.data.id)).toEqual(['N-1']);
    expect(result.waiting.map((card) => card.data.id)).toEqual(['M-1']);
  });

  it('a reply to a message answers it, the entries of the page itself are never cards', () => {
    const { waiting, notices } = work({
      entries: [entry({ id: 'M-1', kind: 'message' }), human({ id: 'O-1', kind: 'followup', replyTo: 'M-1' }), entry({ id: 'E-1', kind: 'edit' })],
    });
    expect([...waiting, ...notices]).toEqual([]);
  });

  it('deleted and archived entries are left out', () => {
    const { waiting } = work({ entries: [entry({ id: 'Q-1', deletedAt: at(2) }), entry({ id: 'Q-2', archivedAt: at(2) }), entry({ id: 'Q-3' })] });
    expect(ids(waiting)).toEqual(['Q-3']);
  });
});

describe('K1 manual steps and tests', { tags: ['critical'] }, () => {
  it('a manual step waits until the owner marks it done, the mark counts until the board applied it', () => {
    const pending = work({ actions: [step({ id: 'H-1' })] });
    expect(ids(pending.waiting)).toEqual(['H-1']);
    const marked = work({ actions: [step({ id: 'H-1' })], entries: [human({ id: 'O-1', kind: 'done', replyTo: 'H-1' })] });
    expect(marked.waiting).toEqual([]);
    const applied = work({
      actions: [step({ id: 'H-1' })],
      entries: [human({ id: 'O-1', kind: 'done', replyTo: 'H-1', appliedAt: at(31), status: 'new' })],
    });
    expect(ids(applied.waiting)).toEqual(['H-1']);
    expect(work({ actions: [step({ id: 'H-1', status: 'done' })] }).waiting).toEqual([]);
  });

  it('the tests of one bundle are ONE card with a count, a bundle of five is not five cards', () => {
    const tests = ['T-5', 'T-2', 'T-10', 'T-1', 'T-3'].map((id, index) => manual({ id, build: 'bundle-4', rank: index === 0 ? 50 : index }));
    const { waiting } = work({ tests });
    expect(waiting).toHaveLength(1);
    const card = waiting[0];
    expect(card?.type).toBe('bundle');
    if (card?.type === 'bundle') expect(card.data.tests.map((test) => test.id)).toEqual(['T-2', 'T-10', 'T-1', 'T-3', 'T-5']);
  });

  it('a test with no bundle stays its own card, automatic and finished tests do not wait on the owner', () => {
    const { waiting } = work({
      tests: [
        manual({ id: 'T-1' }),
        manual({ id: 'T-2', mode: 'auto' }),
        manual({ id: 'T-3', status: 'pass' }),
        manual({ id: 'T-4', archivedAt: at(4) }),
      ],
    });
    expect(waiting.map((card) => [card.type, card.data.id])).toEqual([['test', 'T-1']]);
  });

  it('a closed bundle is out of the feed', () => {
    const { waiting } = work({ tests: [manual({ id: 'T-1', build: 'old' }), manual({ id: 'T-2', build: 'new' })], builds: [{ id: 'old', closedAt: at(3) }, { id: 'new' }] });
    expect(waiting.map((card) => card.data.id)).toEqual(['new']);
  });
});

describe('K1 what each work item needs', { tags: ['critical'] }, () => {
  it('counts open questions, manual steps, tests to run and decisions waiting to be picked up', () => {
    const { needs } = work({
      entries: [
        entry({ id: 'Q-1', item: 'I-1' }),
        entry({ id: 'Q-2', item: 'I-1', options: undefined }),
        entry({ id: 'D-1', kind: 'decision', status: 'pending', author: 'human', item: 'I-1' }),
        entry({ id: 'Q-3', item: 'I-2', status: 'closed' }),
      ],
      actions: [step({ id: 'H-1', item: 'I-1' })],
      tests: [manual({ id: 'T-1', item: 'I-2', build: 'b' }), manual({ id: 'T-2', item: 'I-2', build: 'b' })],
    });
    expect(needs.get('I-1')).toEqual({ questions: 2, decisions: 1, steps: 1, tests: 0 });
    expect(needs.get('I-2')).toEqual({ questions: 0, decisions: 0, steps: 0, tests: 2 });
  });
});

describe('K1 work items and the log', { tags: ['critical'] }, () => {
  const item = (id: string, over: object = {}) => ({ id, title: id, status: 'doing', thread: 'lead', ...over });

  it('shows the open items in manual order and hides done, dropped, archived and deleted ones', () => {
    const { items } = work({
      items: [
        item('I-10', { rank: 3 }),
        item('I-2', { rank: 1 }),
        item('I-3', { status: 'done' }),
        item('I-4', { status: 'dropped' }),
        item('I-5', { archivedAt: at(1) }),
        item('I-6', { deletedAt: at(1) }),
        item('I-1', { rank: 1 }),
      ],
    });
    expect(items.map((doc) => doc.id)).toEqual(['I-1', 'I-2', 'I-10']);
  });

  it('lists the lines of every subject, newest first, each with its subject', () => {
    const { lines } = work({
      log: [
        { subject: 'audit', entries: [{ seq: 1, ts: at(1), thread: 'a', text: 'first' }, { seq: 2, ts: at(8), thread: 'a', text: 'third' }] },
        { subject: 'leads', entries: [{ seq: 1, ts: at(4), thread: 'b', text: 'second' }] },
      ],
    });
    expect(lines.map((line) => [line.text, line.subject])).toEqual([['third', 'audit'], ['second', 'leads'], ['first', 'audit']]);
  });
});

describe('K1 context of a question', { tags: ['critical'] }, () => {
  it('only a written context counts', () => {
    expect(hasContext(entry({ id: 'Q-1', context: 'why' }))).toBe(true);
    expect(hasContext(entry({ id: 'Q-2', context: '  \n ' }))).toBe(false);
    expect(hasContext(entry({ id: 'Q-3' }))).toBe(false);
  });

  it('two paragraphs show and the rest opens on click', () => {
    expect(contextParts('one\n\ntwo\n\nthree\n\nfour')).toEqual({ shown: 'one\n\ntwo', rest: 'three\n\nfour' });
    expect(contextParts('one\n\ntwo')).toEqual({ shown: 'one\n\ntwo', rest: '' });
  });

  it('a context with a code fence stays whole', () => {
    const text = 'before\n\n```\ncode\n\nmore\n```\n\nafter';
    expect(contextParts(text)).toEqual({ shown: text, rest: '' });
  });
});
