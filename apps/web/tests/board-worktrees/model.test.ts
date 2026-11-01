/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import {
  aboutSize, askProblem, batchNotice, canRemove, compareRows, confirmStale, countRows, daysText, entryProblem, filterRows, isNeeding,
  hygieneTag, isSafe, isYoung, leftoverFlags, lostItems, nextPanel, refusedJobs, removalInput, removalNotice, removeLook, sizeText, statusShown,
  summaryParts,
  threadOf,
} from '../../src/screens/board/tabs/worktrees/model';
import { DAY, ago, leftover, metaDoc, oldMeta, rule, worktree } from './fixtures';

describe('sizes and days', () => {
  it('writes megabytes, gigabytes and the small cases like the Node page', () => {
    expect(sizeText(null)).toBe('unknown size');
    expect(sizeText(0.2)).toBe('under 1 MB');
    expect(sizeText(512.4)).toBe('512 MB');
    expect(sizeText(1536)).toBe('1.5 GB');
    expect(aboutSize(2048)).toBe('about 2 GB');
    expect(aboutSize(0.2)).toBe('under 1 MB');
    expect(daysText(1)).toBe('1 day');
    expect(daysText(3)).toBe('3 days');
  });
});

describe('the order and the filters', () => {
  const rows = [
    worktree('active-1', 'active', { ageDays: 0 }),
    worktree('stale-1', 'stale'),
    worktree('merged-1', 'merged', { ageDays: 9, merged: true, unmerged: 0 }),
    worktree('dirty-1', 'dirty-stale', { dirty: 3, ageDays: 5 }),
    worktree('plain-1', 'not-git', { ageDays: null }),
    leftover('left-1'),
  ];

  it('puts the ones that need a decision first, the oldest first, then the rest', () => {
    const names = [...rows].sort(compareRows).map((row) => row.name);
    expect(names).toEqual(['merged-1', 'stale-1', 'dirty-1', 'left-1', 'active-1', 'plain-1']);
  });

  it('counts and filters by need and by leftover', () => {
    expect(countRows(rows)).toEqual({ all: 6, need: 3, left: 1, git: 5 });
    expect(filterRows(rows, 'need').map((row) => row.name)).toEqual(['stale-1', 'merged-1', 'dirty-1']);
    expect(filterRows(rows, 'left').map((row) => row.name)).toEqual(['left-1']);
    expect(filterRows(rows, 'all')).toHaveLength(6);
    expect(isNeeding(worktree('k', 'kept'))).toBe(false);
  });
});

describe('what the owner may remove, and how it reads', () => {
  it('a clean worktree on a branch is safe; uncommitted work or a lone detached HEAD is not', () => {
    expect(isSafe(worktree('a', 'stale'))).toBe(true);
    expect(isSafe(worktree('b', 'stale', { dirty: 2 }))).toBe(false);
    expect(isSafe(worktree('c', 'stale', { detached: true, branch: null, referenced: false, unmerged: 2 }))).toBe(false);
    expect(isSafe(worktree('d', 'stale', { detached: true, branch: null, referenced: true }))).toBe(true);
  });

  it('removable: not a main, not locked, not not-git, error or deleting; a leftover only when measured and plain', () => {
    expect(canRemove(worktree('a', 'stale'))).toBe(true);
    for (const row of [worktree('m', 'stale', { main: true }), worktree('l', 'stale', { locked: 'demo' }), worktree('n', 'not-git'), worktree('e', 'error')]) {
      expect(canRemove(row)).toBe(false);
    }
    expect(canRemove(leftover('ok'))).toBe(true);
    expect(canRemove(leftover('nested', {}, { hasNestedGit: true }))).toBe(false);
    expect(canRemove(leftover('link', {}, { isLink: true }))).toBe(false);
    expect(canRemove(leftover('new', {}, { pending: true }))).toBe(false);
    expect(canRemove(leftover('going', { status: 'deleting' }))).toBe(false);
  });

  it('labels: Remove, Delete now, Delete leftover', () => {
    expect(removeLook(worktree('a', 'stale')).label).toBe('Remove');
    expect(removeLook(worktree('b', 'stale', { dirty: 1 })).label).toBe('Delete now');
    expect(removeLook(leftover('c')).label).toBe('Delete leftover');
  });

  it('lists what a discard loses: files, the unmerged commits with their first subjects, the branch', () => {
    const row = worktree('a', 'stale', { dirty: 2, unmerged: 4, unmergedSubjects: ['wip', 'fix the form', 'more', 'extra'], branch: 'feat/a' });
    expect(lostItems(row)).toEqual([
      '2 uncommitted files (changed or new)',
      'the branch feat/a with its 4 commits not merged into main: "wip", "fix the form", "more" and 1 more, if you also delete it',
    ]);
    const loose = worktree('c', 'stale', { detached: true, branch: null, unmerged: null });
    expect(lostItems(loose)).toEqual(['commits not known to be in the default branch (on a detached HEAD that no branch keeps)']);
  });

  it('a leftover is young under the limit and when its age is unknown', () => {
    expect(isYoung(leftover('a'), 3)).toBe(false);
    expect(isYoung(leftover('b', {}, { ageDays: 0 }), 3)).toBe(true);
    expect(isYoung(leftover('c', {}, { ageDays: null }), 3)).toBe(true);
  });

  it('flags a nested repository, a link, node_modules and a partial or pending measure', () => {
    const flags = leftoverFlags(leftover('x', {}, { hasNestedGit: true, nestedGit: 'inner/.git', isLink: true, nodeModules: true, partial: true }));
    expect(flags.map((flag) => flag.text)).toEqual(['nested git', 'link', 'node_modules', 'partial']);
    expect(flags[0]?.tip).toContain('inner/.git');
    expect(leftoverFlags(leftover('y', {}, { pending: true })).map((flag) => flag.text)).toEqual(['measuring']);
  });
});

describe('the panel and what the owner saw', () => {
  const stale = worktree('a', 'stale');

  it('opens, closes on the same button and switches kind, keeping the typed text between delete and check', () => {
    const open = nextPanel(null, stale, 'delete', null);
    expect(open).toMatchObject({ id: 'id-a', kind: 'delete', text: '', error: null });
    expect(nextPanel(open, stale, 'delete', null)).toBeNull();
    const typed = open ? { ...open, text: 'hello' } : null;
    expect(nextPanel(typed, stale, 'check', null)?.text).toBe('hello');
    expect(nextPanel(typed, stale, 'remove', null)?.text).toBe('');
    expect(nextPanel(typed, worktree('b', 'stale'), 'delete', null)?.text).toBe('');
  });

  it('freezes the HEAD, the dirty count and the discard flag when the confirm opens', () => {
    const dirty = worktree('d', 'stale', { dirty: 2, head: 'beef123' });
    const panel = nextPanel(null, dirty, 'remove', null);
    expect(panel?.seen).toMatchObject({ head: 'beef123', dirty: 2, discard: true });
    expect(panel ? removalInput(dirty, { ...panel, branch: true }) : null).toEqual({ id: 'id-d', branch: true, discard: true, head: 'beef123', dirty: 2 });
  });

  it('a leftover confirm sends the newest change it saw and whether it was young', () => {
    const row = leftover('a', {}, { ageDays: 0 });
    const panel = nextPanel(null, row, 'remove', metaDoc());
    expect(panel ? removalInput(row, panel) : null).toEqual({ id: 'id-a', newestAt: row.leftover?.newestAt, confirmYoung: true });
  });

  it('the confirm is stale when the row moved since it opened, and for a row that can no longer go', () => {
    const row = worktree('a', 'stale');
    const panel = nextPanel(null, row, 'remove', null);
    if (!panel) throw new Error('no panel');
    expect(confirmStale(row, panel)).toBe(false);
    expect(confirmStale({ ...row, dirty: 1 }, panel)).toBe(true);
    expect(confirmStale({ ...row, head: 'other' }, panel)).toBe(true);
    expect(confirmStale({ ...row, main: true }, panel)).toBe(true);
    const lo = leftover('l');
    const leftPanel = nextPanel(null, lo, 'remove', null);
    if (!leftPanel) throw new Error('no panel');
    expect(confirmStale(lo, leftPanel)).toBe(false);
    expect(confirmStale(leftover('l', {}, { newestAt: ago(DAY) }), leftPanel)).toBe(true);
  });
});

describe('the sentences', () => {
  it('the thread comes from the row, else from its rule', () => {
    expect(threadOf(worktree('a', 'stale'), null)).toBe('lead');
    expect(threadOf(worktree('a', 'stale', { thread: undefined, rule: 'W-2' }), metaDoc({ rules: [rule({ id: 'W-2', thread: 'worker' })] }))).toBe('worker');
    expect(threadOf(worktree('a', 'stale', { thread: '' }), metaDoc())).toBe('lead');
    expect(threadOf(worktree('a', 'stale', { thread: undefined }), null)).toBe('');
  });

  it('asks for a thread and keeps the message within 600 characters', () => {
    expect(askProblem('', 'x')).toContain('No thread to tell');
    expect(askProblem('lead', 'x'.repeat(601))).toBe('The message is 601 characters; 600 at most.');
    expect(askProblem('lead', 'x'.repeat(600))).toBeNull();
  });

  it('a conflict or a refusal is a sentence, an applied entry is none', () => {
    const base = { id: 'O-1', entry: null, seq: 1 };
    expect(entryProblem({ ...base, status: 'refused', outcome: 'read only' })).toBe('Refused: read only.');
    expect(entryProblem({ ...base, status: 'conflict', outcome: null })).toContain('Nothing of yours was applied');
    expect(entryProblem({ ...base, status: 'applied', outcome: null })).toBeNull();
  });

  it('a removal says what happened to the folder and the branch, with the backup and its restore command', () => {
    const row = worktree('a', 'stale');
    const done = removalNotice(row, { branch: 'feat/a', deletedBranch: true, branchWhy: 'merged into local master (not pushed)', backup: { bundle: 'B.bundle', patch: 'B.patch', restore: ['git fetch B.bundle'] } });
    expect(done).toEqual({ text: 'Removed a, branch feat/a deleted (merged into local master (not pushed)). Backup:', backup: 'B.bundle', restore: 'git fetch B.bundle' });
    expect(removalNotice(row, { branch: 'a', deletedBranch: false, branchNote: 'not merged' }).text).toBe('Removed a, branch a kept (not merged).');
    expect(removalNotice(row, { prunedOnly: true }).text).toBe('Pruned a.');
  });

  it('the batch answer counts the queued folders and the distinct reasons for the skipped ones', () => {
    const text = batchNotice({ started: 1, skipped: [{ id: 'x', reason: 'nested' }, { id: 'y', reason: 'nested' }] }).text;
    expect(text).toBe('Queued 1 leftover folder for deletion, skipped 2 (nested). Argus deletes them one after another in the background; each row goes when its folder is gone.');
    expect(batchNotice({ started: 3 }).text).toContain('Queued 3 leftover folders for deletion.');
  });

  it('the summary: git worktrees and leftovers apart, gone, rule problems and too many', () => {
    const rows = [worktree('a', 'stale'), leftover('b'), leftover('c')];
    const parts = summaryParts(countRows(rows), metaDoc({ gone: 2, tooMany: true, rules: [rule({ exists: false }), rule({ id: 'W-2', error: 'not a git repository' }), rule({ id: 'W-3' })] }));
    expect(parts).toEqual({
      counts: '1 git worktree, 2 leftover folders',
      need: '1 needs a decision',
      gone: '2 gone (git still lists them)',
      problems: '2 rule problems',
      many: 'Alpha has 3 folders: that is a lot, delete the old leftovers',
    });
    const calm = summaryParts(countRows([worktree('a', 'active')]), metaDoc({ gone: 1 }));
    expect(calm.counts).toBe('1 worktree');
    expect(calm.gone).toBe('1 gone (git still lists it)');
    expect(calm.many).toBeNull();
  });

  it('lists the batch jobs a guard refused in the last hour, minus the dismissed ones', () => {
    const jobs = [
      { id: 'L-1', state: 'refused', name: 'a', finishedAt: ago(60_000) },
      { id: 'L-2', state: 'refused', name: 'b', finishedAt: ago(2 * 3_600_000) },
      { id: 'L-3', state: 'done', name: 'c', finishedAt: ago(60_000) },
      { id: 'L-4', state: 'refused', name: 'd', finishedAt: ago(60_000) },
    ];
    const shown = refusedJobs(oldMeta({ jobs }), new Set(['L-4']), Date.now());
    expect(shown.map((job) => job.id)).toEqual(['L-1']);
  });
});

describe('the state a row shows', () => {
  it('hides the three day "active" status and the staleness of a worktree that is not in review', () => {
    expect(statusShown(worktree('a', 'active'))).toBe(false);
    expect(statusShown(worktree('b', 'stale'))).toBe(true);
    const state = (value: string) => ({ hygiene: { state: value } });
    expect(statusShown(worktree('c', 'stale', state('review')))).toBe(true);
    expect(statusShown(worktree('d', 'stale', state('released')))).toBe(false);
    expect(statusShown(worktree('e', 'merged', state('in-use')))).toBe(false);
    expect(statusShown(worktree('f', 'kept', state('kept')))).toBe(false);
    expect(statusShown(worktree('g', 'leftover', state('released')))).toBe(true);
  });

  it('names the states: in use, released, review and kept (leftover comes from the status)', () => {
    const tag = (state: string, why?: string) => hygieneTag({ state, why, thread: 'lead' });
    expect(tag('in-use', 'its agent A-1 is working').text).toBe('in use');
    expect(tag('released').text).toBe('released');
    expect(tag('review', 'it is locked (x)').text).toBe('review');
    expect(tag('kept').text).toBe('kept');
  });
});
