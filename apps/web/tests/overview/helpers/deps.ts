/* @layer renderer-app @kind test */
import type { DepDoc } from '../../../src/data/deps';
import { minutesAgo } from './fixtures';

const dep = (id: string, extra: Partial<DepDoc> = {}): DepDoc => ({
  id,
  rev: 1,
  createdAt: minutesAgo(600),
  createdBy: 'archipelia/T-2',
  updatedAt: minutesAgo(5),
  state: 'open',
  from: { project: 'archipelia', item: 'I-7', thread: 'T-2' },
  to: { project: 'rotp', item: 'I-12', thread: 'T-1' },
  title: 'Expose the save hook',
  reason: null,
  openedAt: minutesAgo(600),
  resolvedAt: null,
  history: [],
  ...extra,
});

const DEPS: Record<string, DepDoc> = {
  'D-1': dep('D-1', { openedAt: minutesAgo(5000) }),
  'D-2': dep('D-2', { from: { project: 'argus', item: 'I-3', thread: 'T-1' }, to: { project: 'brock', item: 'I-4', thread: null }, openedAt: minutesAgo(120) }),
  'D-3': dep('D-3', { state: 'blocked', reason: 'needs the schema', from: { project: 'rotp', item: 'I-2', thread: 'T-3' }, to: { project: 'argus', item: 'I-9', thread: 'T-1' } }),
  'D-4': dep('D-4', { state: 'done', openedAt: minutesAgo(900), resolvedAt: minutesAgo(30), from: { project: 'brock', item: 'I-1', thread: 'T-1' }, to: { project: 'argus', item: 'I-5', thread: 'T-1' } }),
};

export { DEPS, dep };
