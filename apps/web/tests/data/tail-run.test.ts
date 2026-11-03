/* @layer renderer-app @kind test */
import { describe, expect, it, vi } from 'vitest';
import { tailRun } from '../../src/data/runs/tail-run';
import type { RunLogLine, RunRow } from '../../src/data/runs/runs.type';
import { createFakeSources } from '../helpers/fake-source';

const logFrame = (seq: number) => ({ type: 'log' as const, id: 't1', seq, stream: 'out' as const, at: '2026-11-03T10:00:00Z', text: `line ${seq}` });
const runFrame = (status: string) => ({ type: 'run' as const, id: 't1', run: { id: 'R-1', status } as RunRow });

describe('C6 tailRun', { tags: ['critical'] }, () => {
  it('asks the link for the tail after the given seq', () => {
    const { open, sent } = createFakeSources();
    tailRun(open, { key: 'rotp', runId: 'R-1', after: 4 }, { onLine: vi.fn() });
    expect(sent).toEqual([{ type: 'tail', id: 't1', key: 'rotp', rid: 'R-1', after: 4 }]);
  });

  it('delivers each line once by seq', () => {
    const { open, handle } = createFakeSources();
    const onLine = vi.fn<(line: RunLogLine) => void>();
    tailRun(open, { key: 'rotp', runId: 'R-1', after: 1 }, { onLine });
    for (const seq of [1, 2, 2, 3, 2]) handle.deliver(logFrame(seq));
    expect(onLine.mock.calls.map(([line]) => line.seq)).toEqual([2, 3]);
  });

  it('stops on a final status', () => {
    const { open, handle, sent } = createFakeSources();
    const onStatus = vi.fn();
    tailRun(open, { key: 'rotp', runId: 'R-1' }, { onLine: vi.fn(), onStatus });
    handle.deliver(runFrame('running'));
    expect(sent.filter((message) => message.type === 'untail')).toHaveLength(0);
    handle.deliver(runFrame('done'));
    expect(onStatus).toHaveBeenCalledTimes(2);
    expect(sent.filter((message) => message.type === 'untail')).toEqual([{ type: 'untail', id: 't1' }]);
  });

  it('sends untail when stopped', () => {
    const { open, sent } = createFakeSources();
    const stop = tailRun(open, { key: 'rotp', runId: 'R-1' }, { onLine: vi.fn() });
    stop();
    stop();
    expect(sent.filter((message) => message.type === 'untail')).toEqual([{ type: 'untail', id: 't1' }]);
  });
});
