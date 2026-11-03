/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createMuxRig, hello, settle, sub } from '../helpers/mux-rig';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C1 MuxConnection failures', { tags: ['critical'] }, () => {
  it('reopens with a doubling delay and gives up as unavailable after 3 failures', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    rig.sources[0]?.emit('error');
    expect(rig.mux.getStatus()).toBe('connecting');
    await vi.advanceTimersByTimeAsync(499);
    expect(rig.sources).toHaveLength(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(rig.sources).toHaveLength(2);
    rig.sources[1]?.emit('error');
    await vi.advanceTimersByTimeAsync(999);
    expect(rig.sources).toHaveLength(2);
    await vi.advanceTimersByTimeAsync(1);
    expect(rig.sources).toHaveLength(3);
    rig.sources[2]?.emit('error');
    expect(rig.mux.getStatus()).toBe('unavailable');
    await vi.advanceTimersByTimeAsync(2000);
    expect(rig.sources).toHaveLength(4);
  });

  it('recovers on the next frame and counts failures from zero again', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    for (let attempt = 0; attempt < 3; attempt += 1) {
      rig.sources.at(-1)?.emit('error');
      await vi.advanceTimersByTimeAsync(8000);
    }
    expect(rig.mux.getStatus()).toBe('unavailable');
    hello(rig.sources.at(-1), 'c9');
    expect(rig.mux.getStatus()).toBe('live');
    const count = rig.sources.length;
    rig.sources.at(-1)?.emit('error');
    expect(rig.mux.getStatus()).toBe('connecting');
    await vi.advanceTimersByTimeAsync(500);
    expect(rig.sources).toHaveLength(count + 1);
  });

  it('treats a POST that fails as a stream failure and reopens', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    hello(rig.sources[0], 'c1');
    rig.reply.status = 500;
    rig.mux.send([sub('rotp', 'items')]);
    await settle();
    expect(rig.sources[0]?.closed).toBe(true);
    await vi.advanceTimersByTimeAsync(500);
    expect(rig.sources).toHaveLength(2);
  });

  it('stops retrying after close', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    rig.sources[0]?.emit('error');
    rig.mux.close();
    await vi.advanceTimersByTimeAsync(10_000);
    expect(rig.sources).toHaveLength(1);
  });
});

describe('C1 MuxConnection authorization', { tags: ['critical'] }, () => {
  it('checks /api/health on the first failure only', async () => {
    const rig = createMuxRig();
    rig.mux.open();
    rig.sources[0]?.emit('error');
    await vi.advanceTimersByTimeAsync(500);
    rig.sources[1]?.emit('error');
    await settle();
    expect(rig.health).toHaveBeenCalledTimes(1);
    expect(rig.mux.getStatus()).toBe('connecting');
  });

  it('is unauthorized when the health check answers 401, even after 3 failures', async () => {
    const rig = createMuxRig();
    rig.health.mockResolvedValue(401);
    rig.mux.open();
    rig.sources[0]?.emit('error');
    await settle();
    expect(rig.mux.getStatus()).toBe('unauthorized');
    for (let attempt = 0; attempt < 2; attempt += 1) {
      await vi.advanceTimersByTimeAsync(8000);
      rig.sources.at(-1)?.emit('error');
    }
    expect(rig.mux.getStatus()).toBe('unauthorized');
  });

  it('leaves unauthorized on the next frame', async () => {
    const rig = createMuxRig();
    rig.health.mockResolvedValue(403);
    rig.mux.open();
    rig.sources[0]?.emit('error');
    await settle();
    await vi.advanceTimersByTimeAsync(500);
    hello(rig.sources[1], 'c2');
    expect(rig.mux.getStatus()).toBe('live');
  });
});
