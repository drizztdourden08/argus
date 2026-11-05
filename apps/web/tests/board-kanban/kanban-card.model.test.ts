/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { ItemRef } from '../../src/data/kanban';
import { acSegments, assigneeLook, cardBars, cardChips, cardProgress, parseTrails, stageLabel } from '../../src/screens/board/tabs/kanban/model';
import type { CardData } from '../../src/screens/board/tabs/kanban/model';
import { signalItems } from '../../src/screens/board/tabs/kanban/signals';
import { HOUR, NOW, before, itemRef, threadRef, viewOf } from './kanban-docs';

const criteria = (met: number, total: number) => Array.from({ length: total }, (_, index) => ({ n: index + 1, text: `c${index + 1}`, done: index < met }));

const cardOf = (item: ItemRef, others: ItemRef[] = [], setup: Parameters<typeof viewOf>[1] = {}): CardData => {
  const card = viewOf([item, ...others], { prefs: { showDone: true }, ...setup }).cards.find((candidate) => candidate.id === item.id);
  if (!card) throw new Error(`no card for ${item.id}`);
  return card;
};

describe('I-72 the type chips of a card', { tags: ['critical'] }, () => {
  it('shows the severity of a bug with its meaning as the tooltip', () => {
    const chips = cardChips(itemRef('I-1', { type: 'bug', fields: { severity: 'S2' } }));
    expect(chips).toEqual([{ id: 'severity', text: 'S2', title: 'Severity: S2 (major)' }]);
  });

  it('shows the cost and the risk of a tech debt, and leaves out a field that is not set', () => {
    expect(cardChips(itemRef('I-1', { type: 'debt', fields: { cost: 'M', risk: 'high' } })).map((chip) => chip.text)).toEqual(['cost M', 'risk high']);
    expect(cardChips(itemRef('I-1', { type: 'debt', fields: { cost: 'L' } })).map((chip) => chip.text)).toEqual(['cost L']);
  });

  it('shows the bundle of a test plan and no chip for the types that have none', () => {
    expect(cardChips(itemRef('I-1', { type: 'test-plan', fields: { bundle: 'bundle-16' } })).map((chip) => chip.text)).toEqual(['bundle-16']);
    expect(cardChips(itemRef('I-1', { type: 'epic', fields: { goal: 'x' } }))).toEqual([]);
    expect(cardChips(itemRef('I-1'))).toEqual([]);
  });
});

describe('I-72 the progress of a card', { tags: ['critical'] }, () => {
  it('words the last run of a test plan with its counts and what is left', () => {
    const runs = [
      { n: 2, results: [{ test: 'T-1', status: 'pass' }] },
      { n: 3, results: [{ test: 'T-1', status: 'pass' }, { test: 'T-2', status: 'pass' }, { test: 'T-3', status: 'fail' }] },
    ];
    const plan = itemRef('I-1', { type: 'test-plan', fields: { cases: ['T-1', 'T-2', 'T-3', 'T-4', 'T-5'], runs } });
    expect(cardProgress(cardOf(plan))).toMatchObject([{ met: 3, total: 5, text: 'run 3: 2 pass 1 fail 2 left' }]);
    expect(cardProgress(cardOf(itemRef('I-2', { type: 'test-plan', fields: { cases: ['T-1'] } })))).toEqual([]);
  });
});

describe('I-72 the stage label of a card', { tags: ['critical'] }, () => {
  it('shows the stage in a lane column only when its name differs from the lane', () => {
    expect(stageLabel(itemRef('I-1', { type: 'bug', stage: 'fixing', status: 'doing' }), 'status')).toBe('Fixing');
    expect(stageLabel(itemRef('I-1', { status: 'doing' }), 'status')).toBeNull();
    expect(stageLabel(itemRef('I-1', { type: 'epic', status: 'todo' }), 'status')).toBe('Open');
  });

  it('shows the stage, or the status of an item without one, in the other groupings', () => {
    expect(stageLabel(itemRef('I-1', { type: 'bug', stage: 'fixing', status: 'doing' }), 'thread')).toBe('Fixing');
    expect(stageLabel(itemRef('I-1', { status: 'review' }), 'assignee')).toBe('Review');
  });
});

describe('I-72 the assignee of a card', { tags: ['critical'] }, () => {
  const threads = [threadRef('worker'), threadRef('lead', { role: 'coordinator' })];

  it('looks like the owner with a ring, the lead of the board, another lead or a thread', () => {
    expect(assigneeLook('owner', 'argus', threads)).toMatchObject({ label: 'You', icon: 'user', owner: true });
    expect(assigneeLook('lead', 'argus', threads)).toMatchObject({ initials: 'AR', color: 'blue', tip: 'Assigned to argus lead (coordinator)' });
    expect(assigneeLook('coordinator', 'brock', [])).toMatchObject({ initials: 'BR', color: 'orange' });
    expect(assigneeLook('lead:tessera', 'argus', threads)).toMatchObject({ initials: 'TE', color: 'pink' });
    expect(assigneeLook('worker', 'argus', threads)).toMatchObject({ initials: 'WO', tip: 'Assigned to worker', owner: false });
  });
});

describe('I-72 the signal strip of a card', { tags: ['critical'] }, () => {
  const dep = (n: number) => ({ kind: 'dep-waits', id: `D-${n}`, label: `A very long request title number ${n} that would break a row`, who: 'lead:brock' }) as const;
  const lock = { kind: 'todo', id: 'H-12', label: 'push go', who: 'owner' } as const;
  const idsOf = (card: CardData) => signalItems(card, NOW).map((signal) => signal.id);

  it('makes one chip per dependency target and one for the blocker, each with the avatar of who it waits on', () => {
    const card = cardOf(itemRef('I-1', { blocked: [dep(1), dep(2), dep(3), lock] }));
    const signals = signalItems(card, NOW);
    expect(signals.map((signal) => [signal.id, signal.count])).toEqual([['dependency-brock', undefined], ['blocked', 1]]);
    expect(signals[1]?.label).toBe('Blocked by H-12 (owner): push go');
    expect(signals[0]?.label).toContain('waiting on brock: A very long request title number 2');
    expect(signals[0]?.avatar).toMatchObject({ label: 'brock lead', initials: 'BR' });
    expect(signals[1]?.avatar).toMatchObject({ label: 'You', variant: 'owner' });
  });

  it('makes several dependency chips for several targets', () => {
    const other = { ...dep(4), who: 'lead:rotp' } as const;
    const card = cardOf(itemRef('I-1', { blocked: [dep(1), other] }));
    expect(idsOf(card)).toEqual(['dependency-brock', 'dependency-rotp']);
  });

  it('shows the quiet signal as its hours', () => {
    const card = cardOf(itemRef('I-1', { status: 'doing', updatedAt: before(42 * HOUR) }));
    expect(signalItems(card, NOW).find((signal) => signal.id === 'quiet')?.count).toBe(42);
  });

  it('shows nothing for a settled dependency, which the board no longer exports as blocking', () => {
    const card = cardOf(itemRef('I-1', { waitsOn: { dep: 'D-4', project: 'rotp', item: 'I-12', title: 't', state: 'done', since: before(HOUR), reason: null }, blocked: [] }));
    expect(signalItems(card, NOW)).toEqual([]);
  });

  it('puts needs you first and then each kind in the order of the precedence, with a count and no text', () => {
    const needs = new Map([['I-1', { questions: 1, decisions: 1, todos: 2, tests: 0 }]]);
    const asks = new Map([['I-1', [{ id: 'Q-3', kind: 'question', via: 'item' }, { id: 'D-1', kind: 'decision', via: 'item' }] as const]]);
    const signals = new Map([['I-1', { failing: 2, pending: 1 }]]);
    const item = itemRef('I-1', { status: 'doing', blocked: [dep(1), lock] });
    const card = cardOf(item, [], { needs, asks: new Map([['I-1', [...(asks.get('I-1') ?? [])]]]), signals });
    expect(idsOf(card)).toEqual(['needs-you', 'question', 'todo', 'test-failing', 'dependency-brock', 'blocked', 'test-pending', 'decision']);
    const signals = signalItems(card, NOW);
    expect(signals.find((signal) => signal.id === 'needs-you')?.avatar).toMatchObject({ variant: 'owner' });
    expect(signals.find((signal) => signal.id === 'todo')).toMatchObject({ count: 2, avatar: { variant: 'owner' } });
  });

  it('opens the entry for a single question and the item for several', () => {
    const one = cardOf(itemRef('I-1'), [], { asks: new Map([['I-1', [{ id: 'Q-3', kind: 'question', via: 'item' }]]]), needs: new Map([['I-1', { questions: 1, decisions: 0, todos: 0, tests: 0 }]]) });
    expect(signalItems(one, NOW).find((signal) => signal.id === 'question')?.opens).toEqual({ kind: 'entry', id: 'Q-3' });
    const two = cardOf(itemRef('I-1'), [], { asks: new Map([['I-1', [{ id: 'Q-3', kind: 'question', via: 'item' }, { id: 'Q-4', kind: 'question', via: 'item' }]]]) });
    expect(signalItems(two, NOW).find((signal) => signal.id === 'question')).toMatchObject({ count: 2, opens: { kind: 'item' } });
  });

  it('adds the trail chip after the dependency chips from depth 3 and not before', () => {
    const item = itemRef('I-1', { blocked: [dep(1)] });
    const deep = cardOf(item, [], { trails: new Map([['I-1', { depth: 3, open: 2 }]]) });
    expect(idsOf(deep)).toEqual(['dependency-brock', 'trail']);
    expect(signalItems(deep, NOW)[1]).toMatchObject({ icon: 'signpost', count: 3, label: 'trail 3, 2 open', tone: 'neutral' });
    expect(idsOf(cardOf(item, [], { trails: new Map([['I-1', { depth: 2, open: 1 }]]) }))).toEqual(['dependency-brock']);
    expect(idsOf(cardOf(item))).toEqual(['dependency-brock']);
  });

  it('keeps the conflict first and the archived mark last', () => {
    const notices = [{ id: 'O-1', entryId: 'O-1', bulk: false, kind: 'conflict', op: { kind: 'move', target: { c: 'items', id: 'I-1' } }, result: {} }] as const;
    const item = itemRef('I-1', { archivedAt: before(HOUR), blocked: [lock] });
    const card = cardOf(item, [], { notices: [...notices], filters: { archived: true } });
    expect(idsOf(card)).toEqual(['conflict', 'blocked', 'archived']);
  });
});

describe('I-72 the trails of the board', { tags: ['critical'] }, () => {
  it('reads the trails route, whatever shape comes back, and skips what is not a trail', () => {
    expect(parseTrails({ trails: { 'I-1': { depth: 3, open: 2 }, 'I-2': { depth: 'x' }, 'I-3': 4 } })).toEqual(new Map([['I-1', { depth: 3, open: 2 }]]));
    expect(parseTrails({ 'I-1': { depth: 4 } })).toEqual(new Map([['I-1', { depth: 4, open: 0 }]]));
    expect(parseTrails([{ id: 'I-5', depth: 3, open: 1 }])).toEqual(new Map([['I-5', { depth: 3, open: 1 }]]));
    expect(parseTrails(null)).toEqual(new Map());
    expect(parseTrails({ id: 'entry', status: 'applied' })).toEqual(new Map());
  });
});

describe('I-95 the bars of a card', { tags: ['critical'] }, () => {
  it('draws one segment per criterion, met in success and open in track', () => {
    const bars = cardBars(cardOf(itemRef('I-1', { ac: criteria(1, 3) })));
    expect(acSegments(bars.criteria).map((segment) => segment.tone)).toEqual(['success', 'track', 'track']);
    expect(cardBars(cardOf(itemRef('I-1'))).criteria).toEqual([]);
  });

  it('draws one segment per live child, in the colour of its type when done and track when open', () => {
    const epic = itemRef('I-1', { type: 'epic' });
    const children = [
      itemRef('I-2', { parent: 'I-1', status: 'done', type: 'bug' }),
      itemRef('I-3', { parent: 'I-1' }),
      itemRef('I-4', { parent: 'I-1', status: 'dropped' }),
    ];
    const work = cardBars(cardOf(epic, children)).work;
    expect(work.map((segment) => segment.tone === 'track')).toEqual([false, true]);
    expect(work[0]?.label).toContain('done');
    expect(work[1]?.label).toContain('Item number 3');
  });

  it('adds the dependencies of the item after its children', () => {
    const waits = { dep: 'D-1', project: 'alpha', item: 'Q-7', title: 'Which?', state: 'open' as const, since: before(HOUR), reason: null };
    const done = { ...waits, dep: 'D-2', item: 'I-9', state: 'done' as const };
    const work = cardBars(cardOf(itemRef('I-1', { waitsOn: [waits, done] }))).work;
    expect(work.map((segment) => segment.tone === 'track')).toEqual([true, false]);
  });

  it('shows both bars on a story and none on a test plan', () => {
    const story = itemRef('I-1', { type: 'story', ac: criteria(3, 4) });
    const bars = cardBars(cardOf(story, [itemRef('I-2', { parent: 'I-1', status: 'done' })]));
    expect([bars.work.length, bars.criteria.length]).toEqual([1, 4]);
    expect(cardBars(cardOf(itemRef('I-3', { type: 'test-plan' })))).toEqual({ work: [], criteria: [] });
  });
});
