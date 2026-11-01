/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { finishedRuns, runsOf } from '../../src/data/runs';
import type { RunEntry } from '../../src/data/runs';

const run = (id: string, queuedAt: string, extra: object = {}): RunEntry =>
  ({ id, name: 'launcher-main', status: 'exited', queuedAt, ...extra }) as RunEntry;

describe('K1 the order of runs', { tags: ['critical'] }, () => {
  it('puts the newest queued run first, and the higher run number first between two queued together', () => {
    const same = '2026-10-26T05:37:51.764Z';
    const runs = [run('R-2', same), run('R-10', same), run('R-3', '2026-10-26T23:18:33.444Z'), run('R-1', '2026-10-26T05:14:09.259Z')];
    expect(runsOf(runs, 'launcher-main').map((entry) => entry.id)).toEqual(['R-3', 'R-10', 'R-2', 'R-1']);
  });

  it('lists as finished history only the runs that are over, newest first, at most 20', () => {
    const many = Array.from({ length: 25 }, (_, n) => run(`R-${n + 1}`, `2026-10-26T05:${String(n).padStart(2, '0')}:00.000Z`));
    const live = run('R-99', '2026-10-26T06:00:00.000Z', { status: 'running' });
    const shown = finishedRuns([...many, live]);
    expect(shown).toHaveLength(20);
    expect(shown[0]?.id).toBe('R-25');
    expect(shown.some((entry) => entry.id === 'R-99')).toBe(false);
  });
});
