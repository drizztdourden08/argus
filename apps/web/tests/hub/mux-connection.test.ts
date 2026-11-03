/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { MuxOp } from '../../src/data/hub';
import { createMuxRig, hello, settle, sub } from '../helpers/mux-rig';

const resumeOps: MuxOp[] = [{ op: 'sub', scope: 'rotp', colls: ['items'], since: 4 }, sub('rotp', 'threads')];

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C1 MuxConnection opening', { tags: ['critical'] }, () => {
  it('opens one source on /api/mux and is connecting until the first frame', () => {
    const rig = createMuxRig();
    rig.mux.open();
    rig.mux.open();
    expect(rig.sources.map((source) => source.url)).toEqual(['/api/mux']);
    expect(rig.mux.getStatus()).toBe('connecting');
  });

  it('sends no op before hello and only the resume ops after it', async () => {
    const rig = createMuxRig(resumeOps);
    rig.mux.open();
    rig.mux.send([sub('rotp', 'items')]);
    await settle();
    expect(rig.posts).toEqual([]);
    hello(rig.sources[0], 'c1');
    await settle();
    expect(rig.posts).toEqual([{ conn: 'c1', ops: resumeOps }]);
    expect(rig.mux.getStatus()).toBe('live');
  });

  it('sends nothing for an empty resume list', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    await settle();
    expect(rig.posts).toEqual([]);
  });

  it('hands every frame after hello to onFrame', () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    rig.sources[0]?.emit('ready', { scope: 'rotp', seq: 3, colls: ['items'] });
    rig.sources[0]?.emit('tail-end', { sub: 't:1' });
    expect(rig.frames.map((frame) => frame.event)).toEqual(['ready', 'tail-end']);
  });

  it('closes the source and goes idle on close', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    rig.mux.close();
    rig.mux.send([sub('rotp', 'items')]);
    await settle();
    expect(rig.sources[0]?.closed).toBe(true);
    expect(rig.mux.getStatus()).toBe('idle');
    expect(rig.posts).toEqual([]);
  });
});

describe('C1 MuxConnection batching', { tags: ['critical'] }, () => {
  it('puts the ops of one tick into one POST in order', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    rig.mux.send([sub('rotp', 'items')]);
    rig.mux.send([sub('rotp', 'threads'), { op: 'unsub', scope: 'rotp', colls: ['items'] }]);
    await settle();
    expect(rig.posts).toHaveLength(1);
    expect(rig.posts[0]?.conn).toBe('c1');
    expect(rig.posts[0]?.ops.map((op) => op.op)).toEqual(['sub', 'sub', 'unsub']);
  });

  it('sends the ops of a later tick in a second POST', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    rig.mux.send([sub('rotp', 'items')]);
    await settle();
    rig.mux.send([sub('rotp', 'threads')]);
    await settle();
    expect(rig.posts.map((sent) => sent.ops.length)).toEqual([1, 1]);
  });

  it('splits a tick of more than 50 ops into ordered POSTs of at most 50', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    rig.mux.send(Array.from({ length: 120 }, (_, index) => sub('rotp', `c${index}`)));
    await settle();
    expect(rig.posts.map((sent) => sent.ops.length)).toEqual([50, 50, 20]);
    expect(rig.posts[2]?.ops.at(-1)).toEqual(sub('rotp', 'c119'));
  });

  it('reports the error of one op and keeps going', async () => {
    const rig = createMuxRig();
    rig.reply.results = [{ ok: true }, { error: { code: 'too_many_tabs', message: 'no' } }];
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    const bad: MuxOp = { op: 'presence', scope: 'rotp', thread: 't', tab: 'x' };
    rig.mux.send([sub('rotp', 'items'), bad]);
    await settle();
    expect(rig.onOpError).toHaveBeenCalledWith(bad, { code: 'too_many_tabs', message: 'no' });
    expect(rig.mux.getStatus()).toBe('live');
  });
});

describe('C1 MuxConnection unknown connection', { tags: ['critical'] }, () => {
  it('reopens on a 404 and re-sends the resume ops to the new connection', async () => {
    const rig = createMuxRig(resumeOps);
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    await settle();
    rig.reply.status = 404;
    rig.mux.send([sub('rotp', 'runs')]);
    await settle();
    expect(rig.sources).toHaveLength(2);
    expect(rig.sources[0]?.closed).toBe(true);
    expect(rig.mux.getStatus()).toBe('connecting');
    rig.reply.status = 200;
    hello(rig.sources[1], 'c2');
    await settle();
    expect(rig.posts.at(-1)).toEqual({ conn: 'c2', ops: resumeOps });
    expect(rig.mux.getStatus()).toBe('live');
  });

  it('backs off when the new connection is also unknown', async () => {
    const rig = createMuxRig(resumeOps);
    rig.reply.status = 404;
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    await settle();
    expect(rig.sources).toHaveLength(2);
    hello(rig.sources[1], 'c2');
    await settle();
    expect(rig.sources).toHaveLength(2);
    await vi.advanceTimersByTimeAsync(500);
    expect(rig.sources).toHaveLength(3);
  });

  it('ignores events of a source that was replaced', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    rig.reply.status = 404;
    rig.mux.send([sub('rotp', 'runs')]);
    await settle();
    rig.sources[0]?.emit('ready', { scope: 'rotp', seq: 1, colls: ['runs'] });
    rig.sources[0]?.emit('error');
    expect(rig.frames).toEqual([]);
    expect(rig.sources).toHaveLength(2);
  });
});
