/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { RequestedBy, WaitsOn } from '../../src/data/deps';
import type { DepLink } from '../../src/screens/board/tabs/kanban/model';
import { depLinks, depTags, depView, historyText } from '../../src/screens/board/tabs/kanban/model';
import { NOW, before, itemRef } from './kanban-docs';

const HOUR = 3_600_000;

const waits = (extra: Partial<WaitsOn> = {}): WaitsOn => ({
  dep: 'D-4', project: 'rotp', item: 'I-12', title: 'Expose the save hook', state: 'open', since: before(2 * HOUR), reason: null, ...extra,
});

const asked = (extra: Partial<RequestedBy> = {}): RequestedBy => ({
  dep: 'D-4', project: 'archipelia', item: 'I-7', thread: 'T-2', title: 'Save the game', state: 'open', ...extra,
});

const only = (item: ReturnType<typeof itemRef>): DepLink => {
  const [link] = depLinks(item);
  if (!link) throw new Error('no dependency link');
  return link;
};

describe('K1 the dependency tags of a card', { tags: ['critical'] }, () => {
  it('says waiting on, in info, while the request is opening or open, with the title and the age as the tooltip', () => {
    const [tag] = depTags(itemRef('I-1', { waitsOn: waits() }), NOW);
    expect(tag).toMatchObject({ key: 'waits', color: 'info', label: 'waiting on rotp/I-12', title: 'Expose the save hook. 2h ago' });
    expect(depTags(itemRef('I-1', { waitsOn: waits({ state: 'opening' }) }), NOW)[0]?.label).toBe('waiting on rotp/I-12');
  });

  it('says blocked, in warning, with the reason in the tooltip', () => {
    const [tag] = depTags(itemRef('I-1', { waitsOn: waits({ state: 'blocked', reason: 'needs the schema' }) }), NOW);
    expect(tag).toMatchObject({ color: 'warning', label: 'rotp/I-12 blocked', title: 'Expose the save hook. Reason: needs the schema. 2h ago' });
  });

  it('says done, in success', () => {
    expect(depTags(itemRef('I-1', { waitsOn: waits({ state: 'done' }) }), NOW)[0]).toMatchObject({ color: 'success', label: 'rotp/I-12 done' });
  });

  it('says dropped or gone in neutral, and request cancelled once the request is cancelled', () => {
    expect(depTags(itemRef('I-1', { waitsOn: waits({ state: 'dropped' }) }), NOW)[0]).toMatchObject({ color: 'neutral', label: 'rotp/I-12 dropped' });
    expect(depTags(itemRef('I-1', { waitsOn: waits({ state: 'gone' }) }), NOW)[0]).toMatchObject({ color: 'neutral', label: 'rotp/I-12 gone' });
    expect(depTags(itemRef('I-1', { waitsOn: waits({ state: 'cancelled' }) }), NOW)[0]).toMatchObject({ color: 'neutral', label: 'request cancelled' });
  });

  it('says for, in neutral, on the item that was requested, and request cancelled once it is cancelled', () => {
    expect(depTags(itemRef('I-1', { requestedBy: asked() }), NOW)[0]).toMatchObject({ key: 'asked', color: 'neutral', label: 'for archipelia/I-7' });
    expect(depTags(itemRef('I-1', { requestedBy: asked({ state: 'cancelled' }) }), NOW)[0]?.label).toBe('request cancelled');
  });

  it('gives one tag per side and none for an item with no dependency', () => {
    expect(depTags(itemRef('I-1', { waitsOn: waits(), requestedBy: asked() }), NOW).map((tag) => tag.key)).toEqual(['waits', 'asked']);
    expect(depTags(itemRef('I-1'), NOW)).toEqual([]);
  });
});

describe('K1 the dependency view of the drawer', { tags: ['critical'] }, () => {
  it('reads the other item from the mirror and the thread, the reason and the history from the ledger', () => {
    const view = depView(only(itemRef('I-1', { waitsOn: waits({ state: 'open' }) })), {
      id: 'D-4', state: 'blocked', from: { project: 'archipelia', item: 'I-7', thread: 'T-2' }, to: { project: 'rotp', item: 'I-12', thread: 'T-1' },
      title: 'Expose the save hook', reason: 'needs the schema', openedAt: before(HOUR), resolvedAt: null,
      history: [{ seq: 1, at: before(HOUR), by: 'rotp/T-1', from: 'open', to: 'blocked', reason: 'needs the schema' }],
    });
    expect(view).toMatchObject({ state: 'blocked', thread: 'T-1', reason: 'needs the schema', cancellable: true, project: 'rotp', item: 'I-12' });
    expect(view.history).toHaveLength(1);
  });

  it('falls back on the mirror before the ledger arrives, and allows no cancel once the request is done', () => {
    expect(depView(only(itemRef('I-1', { waitsOn: waits({ state: 'done' }) })), null)).toMatchObject({ state: 'done', thread: null, history: [], cancellable: false });
  });

  it('writes a history line with the age, the author, the move and the reason', () => {
    expect(historyText({ seq: 2, at: before(2 * HOUR), by: 'rotp/T-1', from: 'open', to: 'blocked', reason: 'needs the schema' }, NOW)).toBe('2h ago: rotp/T-1 moved it from open to blocked, needs the schema');
    expect(historyText({ seq: 1, at: before(HOUR), by: 'archipelia/T-2', from: null, to: 'opening', reason: null }, NOW)).toBe('1h ago: archipelia/T-2 moved it from nothing to opening');
  });
});
