/* @layer renderer-app @kind test */
import type { Worktree, WorktreeRule, WorktreesMeta } from '../../src/data/worktrees';
import type { WorktreeRow } from '../../src/screens/board/tabs/worktrees/model';

const MIN = 60_000;
const HOUR = 3_600_000;
const DAY = 86_400_000;
const NOW = Date.parse('2026-10-26T12:00:00.000Z');

type LeftoverFacts = NonNullable<Worktree['leftover']>;

const ago = (ms: number): string => new Date(NOW - ms).toISOString();

const rule = (extra: Partial<WorktreeRule> = {}): WorktreeRule => ({
  id: 'W-1',
  path: 'X:/proj/trees',
  kind: 'folder',
  thread: 'lead',
  note: 'where the lead builds',
  staleDays: null,
  exists: true,
  candidates: 4,
  error: null,
  ...extra,
});

const metaDoc = (extra: Partial<WorktreesMeta> = {}): WorktreesMeta => ({
  name: 'Alpha',
  scannedAt: ago(3 * MIN),
  staleDays: 3,
  rules: [rule()],
  ...extra,
});

const worktree = (name: string, status: string, extra: Partial<Worktree> = {}): WorktreeRow => ({
  id: `id-${name}`,
  path: `X:/proj/trees/${name}`,
  name,
  rule: 'W-1',
  thread: 'lead',
  branch: name,
  detached: false,
  head: 'abc1234',
  subject: `work on ${name}`,
  lastCommitAt: ago(6 * DAY),
  lastTouchedAt: ago(6 * DAY),
  ageDays: 6,
  dirty: 0,
  ahead: 0,
  behind: 0,
  merged: false,
  mergedInto: 'main',
  locked: null,
  main: false,
  status,
  kept: null,
  request: null,
  error: null,
  ...extra,
});

const leftover = (name: string, extra: Partial<Worktree> = {}, facts: Partial<LeftoverFacts> = {}): WorktreeRow =>
  worktree(name, 'leftover', {
    branch: null,
    head: null,
    subject: null,
    lastCommitAt: null,
    lastTouchedAt: ago(12 * DAY),
    ageDays: 12,
    leftover: {
      sizeMB: 1536,
      files: 34210,
      newestAt: ago(12 * DAY),
      ageDays: 12,
      hasNestedGit: false,
      nestedGit: null,
      nodeModules: false,
      isLink: false,
      partial: false,
      pending: false,
      topEntries: ['dist/', 'logs/', 'notes.txt'],
      ...facts,
    },
    ...extra,
  });

const oldMeta = (extra: Partial<WorktreesMeta> = {}): WorktreesMeta =>
  metaDoc({ leftoverOld: { days: 3, count: 2, sizeMB: 2048 }, ...extra });

export { DAY, HOUR, MIN, NOW, ago, leftover, metaDoc, oldMeta, rule, worktree };
