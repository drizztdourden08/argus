/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { askMode, contextParts, hasContext } from '../../src/screens/board/feed-model';
import { at, entry, human, ids, manual, plan, run, step, work } from './feed-docs';

describe('K1 what asks the owner', { tags: ['critical'] }, () => {
  it('a question with options is a decision, one without is free text, an old decision wording is information', () => {
    expect(askMode(entry({ id: 'Q-1' }))).toBe('decide');
    expect(askMode(entry({ id: 'Q-2', options: undefined }))).toBe('answer');
    expect(askMode(entry({ id: 'Q-3', options: [], needs: 'decision' }))).toBe('info');
    expect(askMode(entry({ id: 'N-1', kind: 'notice' }))).toBe('info');
  });

  it('open questions split into decision and question cards, newest first', () => {
    const { asks } = work({
      entries: [
        entry({ id: 'Q-1', createdAt: at(1) }),
        entry({ id: 'Q-2', options: undefined, createdAt: at(9) }),
      ],
    });
    expect(asks.map((card) => [card.type, card.data.id])).toEqual([['question', 'Q-2'], ['decision', 'Q-1']]);
  });

  it('an answered or dismissed question leaves the feed at once, a closed one never shows', () => {
    const { asks } = work({
      entries: [
        entry({ id: 'Q-1' }),
        entry({ id: 'Q-2' }),
        entry({ id: 'Q-3' }),
        entry({ id: 'Q-4', status: 'closed' }),
        human({ id: 'O-1', kind: 'decision', replyTo: 'Q-1' }),
        human({ id: 'O-2', kind: 'ack', replyTo: 'Q-2' }),
      ],
    });
    expect(ids(asks)).toEqual(['Q-3']);
  });

  it('a notice never shows, an open message needs the owner', () => {
    const result = work({
      entries: [entry({ id: 'N-1', kind: 'notice', createdAt: at(3) }), entry({ id: 'M-1', kind: 'message', createdAt: at(4) })],
    });
    expect(ids(result.asks)).toEqual(['M-1']);
  });

  it('a reply to a message answers it, the entries of the page itself are never cards', () => {
    const { asks } = work({
      entries: [entry({ id: 'M-1', kind: 'message' }), human({ id: 'O-1', kind: 'followup', replyTo: 'M-1' }), entry({ id: 'E-1', kind: 'edit' })],
    });
    expect(asks).toEqual([]);
  });

  it('deleted and archived entries are left out', () => {
    const { asks } = work({ entries: [entry({ id: 'Q-1', deletedAt: at(2) }), entry({ id: 'Q-2', archivedAt: at(2) }), entry({ id: 'Q-3' })] });
    expect(ids(asks)).toEqual(['Q-3']);
  });
});

describe('K1 manual steps and tests', { tags: ['critical'] }, () => {
  it('feeds no card to the activity, only the needs of the item', () => {
    const result = work({
      actions: [step({ id: 'H-1', item: 'I-1' })],
      tests: [manual({ id: 'T-1', item: 'I-2' })],
      items: [plan({ id: 'I-5' }), run({ id: 'I-6' })],
    });
    expect(result.asks).toEqual([]);
    expect(result.needs.get('I-1')?.steps).toBe(1);
    expect(result.needs.get('I-2')?.tests).toBe(1);
    expect(result.needs.get('I-5')?.tests).toBe(1);
  });

  it('a manual step counts until the owner marks it done, the mark counts until the board applied it', () => {
    const steps = (entries: ReturnType<typeof human>[], status = 'pending') => work({ actions: [step({ id: 'H-1', item: 'I-1', status })], entries }).needs.get('I-1')?.steps;
    expect(steps([])).toBe(1);
    expect(steps([human({ id: 'O-1', kind: 'done', replyTo: 'H-1' })])).toBeUndefined();
    expect(steps([human({ id: 'O-1', kind: 'done', replyTo: 'H-1', appliedAt: at(31), status: 'new' })])).toBe(1);
    expect(steps([], 'done')).toBeUndefined();
  });

  it('a run waits for the owner when ready for him or reported, a running, passed, lead-held or deleted run does not', () => {
    const items = [
      plan({ id: 'I-5' }),
      run({ id: 'I-6', stage: 'reported', assignee: null }),
      run({ id: 'I-7', stage: 'running' }),
      run({ id: 'I-8', stage: 'passed' }),
      run({ id: 'I-9', assignee: 'lead' }),
      run({ id: 'I-10', deletedAt: at(3) }),
    ];
    expect(work({ items }).needs.get('I-5')?.tests).toBe(1);
  });

  it('automatic, finished, archived and built tests do not wait on the owner', () => {
    const tests = [
      manual({ id: 'T-2', item: 'I-1', mode: 'auto' }),
      manual({ id: 'T-3', item: 'I-1', status: 'pass' }),
      manual({ id: 'T-4', item: 'I-1', archivedAt: at(4) }),
      manual({ id: 'T-5', item: 'I-1', build: 'b-1' }),
    ];
    expect(work({ tests }).needs.get('I-1')).toBeUndefined();
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
      tests: [manual({ id: 'T-1', item: 'I-2' }), manual({ id: 'T-2', item: 'I-2' })],
    });
    expect(needs.get('I-1')).toEqual({ questions: 2, decisions: 1, steps: 1, tests: 0 });
    expect(needs.get('I-2')).toEqual({ questions: 0, decisions: 0, steps: 0, tests: 2 });
  });
});

describe('K1 work items', { tags: ['critical'] }, () => {
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
