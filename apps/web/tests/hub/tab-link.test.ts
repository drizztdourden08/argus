/* @layer renderer-app @kind test */
import { describe, expect, it, vi } from 'vitest';
import type { RunLogLine } from '../../src/data/runs/runs.type';
import { createFakeSources } from '../helpers/fake-source';

const line = (seq: number): RunLogLine => ({ seq, stream: 'out', at: '2026-11-03T10:00:00Z', text: `line ${seq}` });

const logFrame = (id: string, seq: number) => ({ type: 'log' as const, id, ...line(seq) });

describe('C5 TabLink wants and presence', { tags: ['critical'] }, () => {
  it('sends a want once per scope and collection and an unwant when it is dropped', () => {
    const { open, sent } = createFakeSources();
    open.want('rotp', 'items');
    open.want('rotp', 'items');
    open.unwant('rotp', 'items');
    open.unwant('rotp', 'items');
    expect(sent).toEqual([
      { type: 'want', scope: 'rotp', coll: 'items' },
      { type: 'unwant', scope: 'rotp', coll: 'items' },
    ]);
  });

  it('re-announces every want, presence and tail the page holds on hub-hello', () => {
    const { open, handle, sent } = createFakeSources();
    open.want('rotp', 'items');
    open.want('_portfolio', 'cards');
    open.presence('rotp', 'lead');
    open.tail('rotp', 'R-1', 4, { onLine: () => undefined });
    sent.length = 0;
    handle.deliver({ type: 'hub-hello', epoch: 2, v: 1 });
    expect(sent).toEqual([
      {
        type: 'announce',
        tabId: 'tab-test',
        v: 1,
        wants: [
          { scope: 'rotp', coll: 'items' },
          { scope: '_portfolio', coll: 'cards' },
        ],
        presences: [{ scope: 'rotp', thread: 'lead' }],
        tails: [{ id: 't1', key: 'rotp', rid: 'R-1', after: 4 }],
      },
    ]);
  });

  it('releases a presence once and ignores a release that a newer presence replaced', () => {
    const { open, sent } = createFakeSources();
    const first = open.presence('rotp', 'lead');
    const second = open.presence('rotp', 'other');
    first();
    expect(sent.map((message) => message.type)).toEqual(['presence', 'presence']);
    second();
    second();
    expect(sent.map((message) => message.type)).toEqual(['presence', 'presence', 'unpresence']);
  });
});

describe('C5 TabLink tails', { tags: ['critical'] }, () => {
  it('delivers tail lines once by seq to the tail that owns them', () => {
    const { open, handle } = createFakeSources();
    const onLine = vi.fn<(received: RunLogLine) => void>();
    const other = vi.fn<(received: RunLogLine) => void>();
    open.tail('rotp', 'R-1', 2, { onLine });
    open.tail('rotp', 'R-2', 0, { onLine: other });
    handle.deliver(logFrame('t1', 2));
    handle.deliver(logFrame('t1', 3));
    handle.deliver(logFrame('t1', 3));
    expect(onLine.mock.calls.map(([received]) => received.seq)).toEqual([3]);
    expect(other).not.toHaveBeenCalled();
  });

  it('carries the last seq into the next announce', () => {
    const { open, handle, sent } = createFakeSources();
    open.tail('rotp', 'R-1', 0, { onLine: () => undefined });
    handle.deliver(logFrame('t1', 7));
    sent.length = 0;
    handle.deliver({ type: 'hub-hello', epoch: 2, v: 1 });
    expect(sent[0]).toMatchObject({ type: 'announce', tails: [{ id: 't1', after: 7 }] });
  });

  it('ends a tail on tail-end and sends untail once when stopped', () => {
    const { open, handle, sent } = createFakeSources();
    const onEnd = vi.fn();
    const stop = open.tail('rotp', 'R-1', 0, { onLine: () => undefined, onEnd });
    handle.deliver({ type: 'tail-end', id: 't1' });
    stop();
    expect(onEnd).toHaveBeenCalledTimes(1);
    expect(sent.map((message) => message.type)).toEqual(['tail', 'untail']);
  });
});
