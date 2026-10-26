/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { launcherViews, runGroups } from '../../src/screens/board/tabs/builds/model';
import type { LauncherDoc } from '../../src/screens/board/tabs/builds/model';
import { launcherUsage } from '../../src/screens/board/tabs/builds/model/launcher-usage';
import { buildDoc, DAY, defDoc, launcherDoc, runDoc, runningDoc, standaloneDoc } from './fixtures';
import { at, builds, defs, input, launchers, NOW, runs } from './model-docs';

describe('launcherViews', () => {
  it('pins master, then current, then name order, and keeps retired ones apart newest first', () => {
    const views = launcherViews(input({
      launchers: launchers(
        launcherDoc('zeta'), launcherDoc('now-1', { current: true }), launcherDoc('master'),
        launcherDoc('old-a', { retiredAt: at(2 * DAY) }), launcherDoc('old-b', { retiredAt: at(DAY) }),
      ),
    }));
    expect(views.live.map((view) => view.launcher.name)).toEqual(['master', 'now-1', 'zeta']);
    expect(views.retired.map((view) => view.launcher.name)).toEqual(['old-b', 'old-a']);
    expect(views.hasMaster).toBe(true);
  });

  it('says there is no master when none is live, even if a retired one carries the name', () => {
    expect(launcherViews(input({ launchers: launchers(launcherDoc('build-2')) })).hasMaster).toBe(false);
    expect(launcherViews(input({ launchers: launchers(launcherDoc('master', { retiredAt: at(DAY) })) })).hasMaster).toBe(false);
  });

  it('finds the definition of a launcher, its runs newest first and the run shown (live first)', () => {
    const view = launcherViews(input({
      runs: runs(
        runDoc('R-1', 'launcher-master', { queuedAt: at(3 * DAY) }),
        runDoc('R-2', 'launcher-master', { queuedAt: at(DAY) }),
        runningDoc('R-3', 'launcher-master', { queuedAt: at(2 * DAY) }),
        runDoc('R-4', 'launcher-other'),
      ),
    })).live[0];
    expect(view?.def?.name).toBe('launcher-master');
    expect(view?.runs.map((run) => run.id)).toEqual(['R-2', 'R-3', 'R-1']);
    expect(view?.shown?.id).toBe('R-3');
  });

  it('shows the newest finished run when nothing is live, and none without runs', () => {
    const shown = launcherViews(input({ runs: runs(runDoc('R-1', 'launcher-master', { queuedAt: at(DAY) }), runDoc('R-2', 'launcher-master', { queuedAt: at(2 * DAY) })) })).live[0];
    expect(shown?.shown?.id).toBe('R-1');
    expect(launcherViews(input()).live[0]?.shown).toBeNull();
  });

  it('has no definition for a retired launcher, and Play is off', () => {
    const view = launcherViews(input({ launchers: launchers(launcherDoc('old', { retiredAt: at(DAY) })), defs: defs(defDoc('old')) })).retired[0];
    expect(view?.def).toBeNull();
    expect(view?.playable).toBe(false);
  });

  it('turns Play off while the bundle it goes with is a draft, never for master', () => {
    const board = {
      launchers: launchers(launcherDoc('master', { joined: 'b1' }), launcherDoc('wt-1', { joined: 'b1' })),
      defs: defs(defDoc('master'), defDoc('wt-1')),
      builds: builds(buildDoc('b1', { status: 'draft' })),
    };
    const views = launcherViews(input(board)).live;
    expect(views.find((view) => view.launcher.name === 'master')?.playable).toBe(true);
    expect(views.find((view) => view.launcher.name === 'wt-1')?.playable).toBe(false);
    expect(views.find((view) => view.launcher.name === 'wt-1')?.joined?.id).toBe('b1');
  });

  it('lists the open bundles that link a launcher, not the closed ones', () => {
    const view = launcherViews(input({
      builds: builds(buildDoc('a', { launcher: 'master' }), buildDoc('b', { launcher: 'master', closedAt: at(DAY) }), buildDoc('c', { launcher: 'other' })),
    })).live[0];
    expect(view?.uses).toEqual(['a']);
  });
});

describe('launcherUsage', () => {
  const usage = (doc: object, list: object[] = []) => launcherUsage(launchers(doc)[0] as LauncherDoc, runs(...list), NOW);

  it('is none for master, a joined launcher and a retired one', () => {
    expect(usage(launcherDoc('master'))).toBeNull();
    expect(usage(standaloneDoc('x', { standalone: false }))).toBeNull();
    expect(usage(standaloneDoc('x', { retiredAt: at(DAY) }))).toBeNull();
  });

  it('is stale only once unused for more than the window', () => {
    const edge = { useEveryMs: 7 * DAY, lastUsedAt: at(7 * DAY) };
    expect(usage(standaloneDoc('x', edge))?.stale).toBe(false);
    expect(usage(standaloneDoc('x', { ...edge, lastUsedAt: at(7 * DAY + 1000) }))?.stale).toBe(true);
    expect(usage(standaloneDoc('x', { lastUsedAt: at(DAY) }))?.stale).toBe(false);
  });

  it('takes the newest Play run of the launcher as a use', () => {
    const old = standaloneDoc('x', { lastUsedAt: at(20 * DAY) });
    expect(usage(old)?.stale).toBe(true);
    expect(usage(old, [runDoc('R-1', 'launcher-x', { queuedAt: at(DAY) })])?.stale).toBe(false);
    expect(usage(old, [runDoc('R-1', 'launcher-y', { queuedAt: at(DAY) })])?.stale).toBe(true);
  });

  it('is held fresh by a keep that has not ended', () => {
    const base = { lastUsedAt: at(20 * DAY) };
    expect(usage(standaloneDoc('x', { ...base, keptUntil: new Date(NOW + DAY).toISOString() }))?.stale).toBe(false);
    expect(usage(standaloneDoc('x', { ...base, keptUntil: new Date(NOW - DAY).toISOString() }))?.stale).toBe(true);
  });

  it('assumes 7 days and says so when the thread set no useEvery', () => {
    expect(usage(standaloneDoc('x', { useEvery: null, useEveryMs: null, lastUsedAt: at(8 * DAY) }))).toMatchObject({ every: '7d', legacy: true, stale: true });
    expect(usage(standaloneDoc('x', { useEvery: '2w', useEveryMs: 14 * DAY }))).toMatchObject({ every: '2w', legacy: false });
  });
});

describe('runGroups', () => {
  const group = (list: object[], defList: object[] = [defDoc('master')]) => runGroups(runs(...list), defs(...defList));

  it('splits live from finished, newest first, ties by run number', () => {
    const groups = group([
      runDoc('R-9', 'launcher-master', { queuedAt: at(DAY) }),
      runDoc('R-10', 'launcher-master', { queuedAt: at(DAY) }),
      runningDoc('R-3', 'launcher-master', { queuedAt: at(2 * DAY) }),
      runDoc('R-1', 'launcher-master', { queuedAt: at(3 * DAY) }),
    ]);
    expect(groups.live.map((run) => run.id)).toEqual(['R-3']);
    expect(groups.finished.map((run) => run.id)).toEqual(['R-10', 'R-9', 'R-1']);
  });

  it('keeps runs of launcher and bundle definitions and of launcher-/bundle- names, nothing else', () => {
    const groups = group(
      [runDoc('R-1', 'launcher-gone'), runDoc('R-2', 'bundle-b1'), runDoc('R-3', 'npm-test'), runDoc('R-4', 'custom-play')],
      [defDoc('master'), { name: 'custom-play', launcher: 'custom', cmd: 'x', hand: true }, { name: 'npm-test', cmd: 'npm test', hand: false }],
    );
    expect(groups.finished.map((run) => run.id).sort()).toEqual(['R-1', 'R-2', 'R-4']);
  });

  it('keeps the last 20 finished runs', () => {
    const many = Array.from({ length: 30 }, (_, index) => runDoc(`R-${index + 1}`, 'launcher-master', { queuedAt: at((40 - index) * 1000) }));
    const groups = group(many);
    expect(groups.finished).toHaveLength(20);
    expect(groups.finished[0]?.id).toBe('R-30');
  });
});
