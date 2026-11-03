/* @layer renderer-app @kind test */
import { describe, expect, it, vi } from 'vitest';
import { ScopeStream } from '../../src/data/stream';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';

const connected = (...colls: string[]) => {
  const fake = createFakeSources();
  const stream = new ScopeStream('rotp', fake.open);
  const listener = vi.fn();
  for (const coll of colls) stream.subscribe(coll, listener);
  const wire = () => fake.sent.map((message) => (message.type === 'want' || message.type === 'unwant' ? `${message.type} ${message.coll}` : message.type));
  return { stream, fake, listener, wire };
};

const change = (seq: number, ...changes: object[]) => ({ scope: 'rotp', seq, changes });

describe('K1 ScopeStream wants', { tags: ['critical'] }, () => {
  it('sends one want per collection on the first subscribe and no url or timer', () => {
    const { stream, wire, fake } = connected('items', 'threads');
    stream.subscribe('items', () => undefined);
    expect(wire()).toEqual(['want items', 'want threads']);
    expect(fake.sources).toHaveLength(1);
    expect(fake.sources[0]?.url).toBe('/api/p/rotp/stream?c=items,threads');
  });

  it('adds a collection with one want and reopens nothing', () => {
    const { stream, wire, fake } = connected('items');
    stream.subscribe('tests', () => undefined);
    expect(wire()).toEqual(['want items', 'want tests']);
    expect(fake.sources).toHaveLength(1);
    expect(fake.sources[0]?.closed).toBe(false);
  });

  it('sends unwant only when the last listener of a collection leaves', () => {
    const { stream, wire } = connected('items');
    const first = stream.subscribe('tests', () => undefined);
    const second = stream.subscribe('tests', () => undefined);
    first();
    expect(wire()).toEqual(['want items', 'want tests']);
    second();
    expect(wire()).toEqual(['want items', 'want tests', 'unwant tests']);
  });
});

describe('K1 ScopeStream status', { tags: ['critical'] }, () => {
  it('is idle, then connecting once subscribed, then follows the hub', () => {
    const { stream, fake } = connected();
    expect(stream.getStatus()).toBe('idle');
    stream.subscribe('items', () => undefined);
    expect(stream.getStatus()).toBe('connecting');
    fake.handle.deliver({ type: 'status', scope: 'rotp', status: 'live' });
    expect(stream.getStatus()).toBe('live');
    fake.handle.deliver({ type: 'status', scope: 'rotp', status: 'unavailable' });
    expect(stream.getStatus()).toBe('unavailable');
  });

  it('is live after a ready frame and idle once the last listener leaves', () => {
    const { stream, fake } = connected();
    const leave = stream.subscribe('items', () => undefined);
    fake.sources[0]?.emit('ready', { seq: 5 });
    expect(stream.getStatus()).toBe('live');
    leave();
    expect(fake.sources[0]?.closed).toBe(true);
    expect(stream.getStatus()).toBe('idle');
  });

  it('tells status listeners once per change and ignores a status for another scope', () => {
    const { stream, fake } = connected('items');
    const listener = vi.fn();
    stream.subscribeStatus(listener);
    fake.handle.deliver({ type: 'status', scope: 'rotp', status: 'live' });
    fake.handle.deliver({ type: 'status', scope: 'rotp', status: 'live' });
    fake.handle.deliver({ type: 'status', scope: 'other', status: 'unavailable' });
    expect(listener).toHaveBeenCalledTimes(1);
    expect(stream.getStatus()).toBe('live');
  });
});

describe('K1 ScopeStream state', { tags: ['critical'] }, () => {
  it('keeps a snapshot per collection and tells only that collection', () => {
    const { stream, fake, listener } = connected('items');
    stream.subscribe('threads', () => undefined);
    fake.sources[0]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { id: 'I-1' } }, 5));
    expect(listener).toHaveBeenCalledTimes(1);
    expect(stream.getCollection('items').docs.get('I-1')).toEqual({ id: 'I-1' });
    expect(stream.getCollection('threads').ready).toBe(false);
  });

  it('reads a snapshot doc that carries its fields flat', () => {
    const { stream, fake } = connected('items');
    fake.sources[0]?.emit('snapshot', { scope: 'rotp', seq: 1, coll: 'items', docs: [{ id: 'I-2', title: 'flat' }] });
    expect(stream.getCollection('items').docs.get('I-2')).toMatchObject({ title: 'flat' });
  });

  it('applies set and delete changes and replaces the collection state', () => {
    const { stream, fake, listener } = connected('items');
    fake.sources[0]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { id: 'I-1', title: 'one' } }));
    const before = stream.getCollection('items');
    fake.sources[0]?.emit('changes', change(11, { coll: 'items', id: 'I-2', op: 'set', data: { id: 'I-2' } }, { coll: 'items', id: 'I-1', op: 'delete' }));
    expect(stream.getCollection('items')).not.toBe(before);
    expect([...stream.getCollection('items').docs.keys()]).toEqual(['I-2']);
    expect(listener).toHaveBeenCalledTimes(2);
  });

  it('ignores frames of another scope', () => {
    const { stream, fake } = connected('items');
    fake.handle.deliver({ type: 'snapshot', ...snapshot('other', 'items', { 'I-1': { id: 'I-1' } }) });
    expect(stream.getCollection('items').ready).toBe(false);
  });

  it('drops its state on reset, tells the listeners and sends nothing', () => {
    const { stream, fake, listener, wire } = connected('items');
    fake.sources[0]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { id: 'I-1' } }, 9));
    listener.mockClear();
    fake.sources[0]?.emit('reset', { reason: 'resync' });
    expect(stream.getCollection('items').ready).toBe(false);
    expect(listener).toHaveBeenCalledTimes(1);
    expect(wire()).toEqual(['want items']);
  });
});
