/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ScopeStream } from '../../src/data/stream';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';

const flush = () => Promise.resolve();

const connected = async (...colls: string[]) => {
  const fake = createFakeSources();
  const stream = new ScopeStream('rotp', fake.open);
  const listener = vi.fn();
  for (const coll of colls) stream.subscribe(coll, listener);
  await flush();
  return { stream, sources: fake.sources, listener };
};

const change = (seq: number, ...changes: object[]) => ({ scope: 'rotp', seq, changes });

beforeEach(() => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('{}', { status: 200 })));
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('K1 ScopeStream connection', { tags: ['critical'] }, () => {
  it('opens one source for the union of the collections once subscribed', async () => {
    const { stream, sources } = await connected('items', 'threads');
    expect(sources).toHaveLength(1);
    expect(sources[0]?.url).toBe('/api/p/rotp/stream?c=items,threads');
    expect(stream.getStatus()).toBe('connecting');
  });

  it('is live after ready and idle once the last listener leaves', async () => {
    const { stream, sources } = await connected();
    const leave = stream.subscribe('items', () => undefined);
    await flush();
    sources[0]?.emit('ready', { seq: 5 });
    expect(stream.getStatus()).toBe('live');
    leave();
    await flush();
    expect(sources[0]?.closed).toBe(true);
    expect(stream.getStatus()).toBe('idle');
  });

  it('reopens with the union when a collection is added', async () => {
    const { stream, sources } = await connected('items');
    stream.subscribe('tests', () => undefined);
    await flush();
    expect(sources[0]?.closed).toBe(true);
    expect(sources[1]?.url).toBe('/api/p/rotp/stream?c=items,tests');
    sources[0]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { id: 'I-1' } }));
    expect(stream.getCollection('items').ready).toBe(false);
  });
});

describe('K1 ScopeStream state', { tags: ['critical'] }, () => {
  it('keeps a snapshot per collection and tells only that collection', async () => {
    const { stream, sources, listener } = await connected('items');
    stream.subscribe('threads', () => undefined);
    await flush();
    sources[1]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { id: 'I-1' } }, 5));
    expect(listener).toHaveBeenCalledTimes(1);
    expect(stream.getCollection('items').docs.get('I-1')).toEqual({ id: 'I-1' });
    expect(stream.getCollection('threads').ready).toBe(false);
  });

  it('reads a snapshot doc that carries its fields flat', async () => {
    const { stream, sources } = await connected('items');
    sources[0]?.emit('snapshot', { scope: 'rotp', seq: 1, coll: 'items', docs: [{ id: 'I-2', title: 'flat' }] });
    expect(stream.getCollection('items').docs.get('I-2')).toMatchObject({ title: 'flat' });
  });

  it('applies set and delete changes and replaces the collection state', async () => {
    const { stream, sources, listener } = await connected('items');
    sources[0]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { id: 'I-1', title: 'one' } }));
    const before = stream.getCollection('items');
    sources[0]?.emit('changes', change(11, { coll: 'items', id: 'I-2', op: 'set', data: { id: 'I-2' } }, { coll: 'items', id: 'I-1', op: 'delete' }));
    expect(stream.getCollection('items')).not.toBe(before);
    expect([...stream.getCollection('items').docs.keys()]).toEqual(['I-2']);
    expect(listener).toHaveBeenCalledTimes(2);
  });

  it('drops its state on reset and asks for a full snapshot', async () => {
    const { stream, sources } = await connected('items');
    sources[0]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { id: 'I-1' } }, 9));
    sources[0]?.emit('reset', { reason: 'resync' });
    expect(stream.getCollection('items').ready).toBe(false);
    expect(sources[1]?.url).toBe('/api/p/rotp/stream?c=items');
  });
});

describe('S3 ScopeStream recovery', { tags: ['standard'] }, () => {
  it('resumes from the last seq after an error', async () => {
    vi.useFakeTimers();
    const { sources } = await connected('items');
    sources[0]?.emit('snapshot', snapshot('rotp', 'items', {}, 40));
    sources[0]?.emit('error');
    expect(sources[0]?.closed).toBe(true);
    vi.advanceTimersByTime(500);
    expect(sources[1]?.url).toBe('/api/p/rotp/stream?c=items&since=40');
  });

  it('goes unavailable after three failures and recovers on the next event', async () => {
    vi.useFakeTimers();
    const { stream, sources } = await connected('items');
    for (let attempt = 0; attempt < 3; attempt += 1) {
      sources.at(-1)?.emit('error');
      vi.advanceTimersByTime(8000);
    }
    expect(stream.getStatus()).toBe('unavailable');
    sources.at(-1)?.emit('snapshot', snapshot('rotp', 'items', {}));
    expect(stream.getStatus()).toBe('live');
  });

  it('reports unauthorized when the health check answers 401', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('', { status: 401 })));
    const { stream, sources } = await connected('items');
    sources[0]?.emit('error');
    await vi.waitFor(() => expect(stream.getStatus()).toBe('unauthorized'));
  });
});
