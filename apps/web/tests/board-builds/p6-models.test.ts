/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { itemGroups } from '../../src/screens/board/item-groups/model';
import { buildCode, buildSignals, launcherViews } from '../../src/screens/board/tabs/builds/model';
import type { LauncherView } from '../../src/screens/board/tabs/builds/model';
import { checksProgress, prSignals } from '../../src/screens/board/tabs/changes/model';
import { shippedOf, releaseCode } from '../../src/screens/releases/model';
import { checkDocs } from '../../src/shell/records/pane/model/check-docs';
import { releaseDocs } from '../../src/shell/records/pane/model/release-docs';
import { itemRef } from '../board-kanban/kanban-docs';
import { DAY, launcherDoc } from './fixtures';
import { at, input, launchers } from './model-docs';
import { channelOf, checkRunOf, prOf, versionOf } from './p6-docs';

interface Row {
  id: string;
  item: string | null;
}

const rows: Row[] = [
  { id: 'a', item: 'I-1' },
  { id: 'b', item: 'I-1' },
  { id: 'c', item: 'I-2' },
  { id: 'd', item: null },
  { id: 'e', item: 'I-9' },
  { id: 'f', item: 'I-3' },
];

const items = [itemRef('I-1', { rank: 20 }), itemRef('I-2', { rank: 10 }), itemRef('I-3', { status: 'done' })];

const ids = (list: Row[]): string[] => list.map((row) => row.id);

const signalIds = (views: readonly LauncherView[], name: string): string[] =>
  views.filter((view) => view.launcher.name === name).flatMap((view) => buildSignals(view).map((signal) => signal.id));

describe('I-85 P6 the rows grouped by their item', { tags: ['critical'] }, () => {
  const result = itemGroups(rows, items, (row) => row.item);

  it('groups the rows of an open item by rank', () => {
    expect(result.groups.map((group) => group.item.id)).toEqual(['I-2', 'I-1']);
    expect(ids(result.groups[1]?.rows ?? [])).toEqual(['a', 'b']);
  });

  it('puts a row with no item, or an item the board does not know, in the no-item group', () => {
    expect(ids(result.none).sort()).toEqual(['d', 'e']);
  });

  it('keeps the rows of a closed item in the closed section, apart from the open ones', () => {
    expect(result.closed.map((group) => group.item.id)).toEqual(['I-3']);
    expect(ids(result.closed[0]?.rows ?? [])).toEqual(['f']);
  });

  it('treats a deleted item as unknown', () => {
    const gone = itemGroups(rows, [itemRef('I-1', { deletedAt: at(DAY) })], (row) => row.item);
    expect(gone.groups).toHaveLength(0);
    expect(gone.none).toHaveLength(rows.length);
  });
});

describe('I-85 P6 the build record', { tags: ['critical'] }, () => {
  it('shows a build by its name under the BL code', () => {
    expect(buildCode('web')).toBe('BL-web');
  });

  it('flags a running build, a build to relink and a build with no item', () => {
    const views = launcherViews(input({ launchers: launchers(launcherDoc('guess', { for: 'I-1', relink: true }), launcherDoc('loose'), launcherDoc('master', { master: true })) }));
    expect(signalIds(views.live, 'guess')).toEqual(['relink']);
    expect(signalIds(views.live, 'loose')).toEqual(['no-item']);
    expect(signalIds(views.live, 'master')).toEqual([]);
  });

  it('never flags nor plays a closed build', () => {
    const views = launcherViews(input({ launchers: launchers(launcherDoc('old', { for: 'I-1', retiredAt: at(DAY), retiredByItem: 'I-1', stage: 'closed' })) }));
    const closed = views.retired[0];
    expect(closed?.playable).toBe(false);
    expect(closed && buildSignals(closed)).toEqual([]);
  });
});

describe('I-85 P6 the change record', { tags: ['standard'] }, () => {
  const failing = [checkRunOf('C-1', { status: 'fail', steps: [{ name: 'lint', command: 'pnpm lint', exitCode: 1, durationMs: 1500, required: true }] })];
  const passing = [checkRunOf('C-2')];

  it('counts the steps that passed in the newest run', () => {
    expect(checksProgress(passing)).toMatchObject({ passed: 2, total: 2, failed: false });
    expect(checksProgress(failing)?.failed).toBe(true);
    expect(checksProgress(null)).toBeNull();
  });

  it('signals a failing check or a failed replay on an open pull request only', () => {
    const open = prOf('PR-1');
    expect(prSignals(open, failing).map((signal) => signal.id)).toEqual(['failing']);
    const replayed = { ...open, replay: { state: 'failed' as const, at: null, url: null } };
    expect(prSignals(replayed, null).map((signal) => signal.id)).toEqual(['conflict']);
    const merged = { ...open, status: 'merged' as const };
    expect(prSignals(merged, failing)).toEqual([]);
  });

  it('writes a check run as a pane document that points at its pull request', () => {
    const docs = checkDocs(passing);
    expect(docs[0]).toMatchObject({ id: 'C-2', status: 'pass', change: 'PR-1' });
    expect(String(docs[0]?.name)).toContain('pr on ');
  });
});

describe('I-85 P6 the release record', { tags: ['critical'] }, () => {
  const version = versionOf('1.2.0', { items: ['I-1', 'I-9'], prs: [{ number: 7, commit: 'abcdef1234567' }, { number: 8, commit: 'ffffffffffff' }] });
  const local = prOf('PR-4', { mergeCommit: 'abcdef1234567890' });

  it('writes the code of a release, with the project in front on the overview', () => {
    expect(releaseCode('rotp', '1.2.0', false)).toBe('V-1.2.0');
    expect(releaseCode('rotp', '1.2.0', true)).toBe('rotp/V-1.2.0');
  });

  it('finds the items a release ships, keeps the unknown ones, and matches a pull request to its local one by merge commit', () => {
    const shipped = shippedOf(version, { items: [itemRef('I-1')], prs: [local] });
    expect(shipped.items.map((item) => item.id)).toEqual(['I-1']);
    expect(shipped.unknown).toEqual(['I-9']);
    expect(shipped.prs.map((entry) => entry.local?.id ?? null)).toEqual(['PR-4', null]);
  });

  it('says nothing is shipped when the release lists nothing', () => {
    const shipped = shippedOf(versionOf('1.0.0'), { items: [], prs: [] });
    expect(shipped).toEqual({ items: [], unknown: [], prs: [] });
  });

  it('writes a release as a pane document, once per version', () => {
    const docs = releaseDocs([channelOf('stable', '1.2.0', [version]), channelOf('beta', '1.2.0', [version])]);
    expect(docs).toHaveLength(1);
    expect(docs[0]).toMatchObject({ id: '1.2.0', channel: 'stable', packages: 1, files: 1, items: ['I-1', 'I-9'], prs: ['#7', '#8'] });
  });
});
