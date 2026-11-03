/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { groupByItem, launcherViews } from '../../src/screens/board/tabs/builds/model';
import { itemRef } from '../board-kanban/kanban-docs';
import { DAY, launcherDoc } from './fixtures';
import { at, input, launchers } from './model-docs';

const groups = (docs: object[], items = [itemRef('I-1'), itemRef('I-2')]) =>
  groupByItem(launcherViews(input({ launchers: launchers(...docs) })), items);

const names = (views: { launcher: { name: string } }[]): string[] => views.map((view) => view.launcher.name);

describe('P13 builds grouped by their item', { tags: ['critical'] }, () => {
  it('puts the master build first in its own section, with no link needed', () => {
    const result = groups([launcherDoc('web', { for: 'I-1' }), launcherDoc('master', { master: true })]);
    expect(names(result.master)).toEqual(['master']);
    expect(names(result.items[0]?.views ?? [])).toEqual(['web']);
    expect(result.needsLink).toHaveLength(0);
  });

  it('knows the master by its name when the server did not say', () => {
    expect(names(groups([launcherDoc('master')]).master)).toEqual(['master']);
  });

  it('groups the other live builds under their open items by rank, and drops items with no build', () => {
    const items = [itemRef('I-1', { rank: 20 }), itemRef('I-2', { rank: 10 }), itemRef('I-3', { rank: 5 })];
    const result = groups([launcherDoc('a', { for: 'I-1' }), launcherDoc('b', { for: 'I-2' }), launcherDoc('c', { for: 'I-2' })], items);
    expect(result.items.map((group) => group.item.id)).toEqual(['I-2', 'I-1']);
    expect(names(result.items[0]?.views ?? [])).toEqual(['b', 'c']);
  });

  it('needs a link: a live build without for, one marked relink, and one whose item is not open', () => {
    const items = [itemRef('I-1'), itemRef('I-2', { status: 'done' })];
    const result = groups(
      [launcherDoc('loose'), launcherDoc('guess', { for: 'I-1', relink: true }), launcherDoc('stale', { for: 'I-2' }), launcherDoc('lost', { for: 'I-9' }), launcherDoc('fine', { for: 'I-1' })],
      items,
    );
    expect(names(result.needsLink)).toEqual(['guess', 'loose', 'lost', 'stale']);
    expect(names(result.items[0]?.views ?? [])).toEqual(['fine']);
  });

  it('keeps the closed builds apart, newest first, whatever item they served', () => {
    const result = groups([
      launcherDoc('web', { for: 'I-1' }),
      launcherDoc('old-a', { for: 'I-2', retiredAt: at(2 * DAY), retiredByItem: 'I-2', stage: 'closed' }),
      launcherDoc('old-b', { for: 'I-1', retiredAt: at(DAY), retiredByItem: 'I-1', stage: 'closed' }),
    ]);
    expect(names(result.closed)).toEqual(['old-b', 'old-a']);
    expect(names(result.items.flatMap((group) => group.views))).toEqual(['web']);
  });

  it('never makes a closed build playable', () => {
    const view = launcherViews(input({ launchers: launchers(launcherDoc('web', { for: 'I-1', stage: 'closed' })) })).live[0];
    expect(view?.playable).toBe(false);
  });
});
