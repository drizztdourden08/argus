/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { ItemRef } from '../../src/data/kanban';
import { ATTENTION, leadTone, signalsMap } from '../../src/screens/board/tabs/kanban/attention';
import type { Signals } from '../../src/screens/board/tabs/kanban/attention';
import type { AskLink, Needs } from '../../src/screens/board/tabs/kanban/model';
import { HOUR, before, itemRef, viewOf } from './kanban-docs';

const none = new Set<string>();

const dep = { dep: 'D-4', project: 'rotp', item: 'I-12', title: 'Expose the hook', state: 'open', since: before(HOUR), reason: null } as const;

const kindsOf = (item: ItemRef, setup: { needs?: Partial<Needs>; asks?: AskLink[]; signals?: Partial<Signals> } = {}) => {
  const needs = new Map([[item.id, { questions: 0, decisions: 0, todos: 0, tests: 0, ...setup.needs }]]);
  const asks = new Map([[item.id, setup.asks ?? []]]);
  const signals = new Map([[item.id, { failing: 0, pending: 0, agents: 0, ...setup.signals }]]);
  return viewOf([item], { needs, asks, signals }).cards[0].attention;
};

describe('I-69 which attention kinds a card carries', { tags: ['critical'] }, () => {
  it('maps each source to its kind', () => {
    const ask = (kind: 'question' | 'decision'): AskLink[] => [{ id: 'Q-1', kind, via: 'item' }];
    expect(kindsOf(itemRef('I-1'), { asks: ask('question') })).toEqual(['question']);
    expect(kindsOf(itemRef('I-1'), { asks: ask('decision') })).toEqual(['decision']);
    expect(kindsOf(itemRef('I-1'), { needs: { todos: 1 } })).toEqual(['todo']);
    expect(kindsOf(itemRef('I-1'), { signals: { failing: 1 } })).toEqual(['test-failing']);
    expect(kindsOf(itemRef('I-1'), { signals: { pending: 1 } })).toEqual(['test-pending']);
    expect(kindsOf(itemRef('I-1'), { signals: { agents: 1 } })).toEqual(['agent']);
    expect(kindsOf(itemRef('I-1', { waitsOn: dep }))).toEqual(['dependency']);
    expect(kindsOf(itemRef('I-1', { requestedBy: { dep: 'D-5', project: 'a', item: 'I-7', thread: 'T-2', title: 't', state: 'blocked' } }))).toEqual(['dependency']);
    expect(kindsOf(itemRef('I-1', { blocker: 'external: legal, the contract' }))).toEqual(['blocked']);
    expect(kindsOf(itemRef('I-1', { status: 'doing', updatedAt: before(5 * HOUR) }))).toEqual(['quiet']);
  });

  it('shows nothing for a finished dependency, a plain request or a blocker that names a live ask', () => {
    expect(kindsOf(itemRef('I-1', { waitsOn: { ...dep, state: 'done' } }))).toEqual([]);
    expect(kindsOf(itemRef('I-1', { requestedBy: { dep: 'D-5', project: 'a', item: 'I-7', thread: 'T-2', title: 't', state: 'open' } }))).toEqual([]);
    expect(kindsOf(itemRef('I-1', { blocker: 'Q-4' }), { asks: [{ id: 'Q-4', kind: 'question', via: 'blocker' }] })).toEqual(['question']);
  });

  it('puts several kinds in precedence order, whatever order the sources came in (a card that waits on you is never quiet)', () => {
    const asks: AskLink[] = [{ id: 'D-1', kind: 'decision', via: 'item' }, { id: 'Q-1', kind: 'question', via: 'item' }];
    const item = itemRef('I-1', { status: 'doing', updatedAt: before(5 * HOUR), blocker: 'external: legal', waitsOn: dep });
    const signals = { failing: 1, pending: 1, agents: 1 };
    const all = ATTENTION.map((spec) => spec.kind);
    expect(kindsOf(item, { asks, signals })).toEqual(all.filter((kind) => kind !== 'todo'));
    expect(kindsOf(item, { asks, signals, needs: { todos: 1 } })).toEqual(all.filter((kind) => kind !== 'quiet'));
  });
});

describe('I-69 the signals of tests and agents', { tags: ['critical'] }, () => {
  const test = (id: string, status: string, extra: Record<string, unknown> = {}) => ({ id, item: 'I-1', status, ...extra });

  it('counts failing, partial and blocked tests as failing and a pending one as pending', () => {
    const map = signalsMap({ tests: [test('T-1', 'fail'), test('T-2', 'partial'), test('T-3', 'blocked'), test('T-4', 'pending'), test('T-5', 'pass')], agents: [], closedBuilds: none });
    expect(map.get('I-1')).toEqual({ failing: 3, pending: 1, agents: 0 });
  });

  it('ignores tests of a closed build, archived or deleted tests, and agents that are not working', () => {
    const map = signalsMap({
      tests: [test('T-1', 'fail', { build: 'bundle-1' }), test('T-2', 'fail', { archivedAt: before(HOUR) }), test('T-3', 'pending', { deletedAt: before(HOUR) })],
      agents: [{ item: 'I-1', status: 'done' }, { item: 'I-1', status: 'working', deletedAt: before(HOUR) }, { item: 'I-2', status: 'working' }],
      closedBuilds: new Set(['bundle-1']),
    });
    expect(map.get('I-1')).toBeUndefined();
    expect(map.get('I-2')).toEqual({ failing: 0, pending: 0, agents: 1 });
  });
});

describe('I-69 the edge colour of a card', { tags: ['critical'] }, () => {
  it('takes the colour of the first kind, and none without a kind', () => {
    expect(leadTone(['test-failing', 'agent'])).toBe('danger');
    expect(leadTone(['dependency', 'agent'])).toBe('warning');
    expect(leadTone(['test-pending', 'quiet'])).toBe('info');
    expect(leadTone(['agent'])).toBe('success');
    expect(leadTone([])).toBeUndefined();
  });
});
