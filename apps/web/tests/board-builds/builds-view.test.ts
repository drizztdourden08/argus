/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { BUILD_FILTERS, buildsView, groupByKind, launcherViews } from '../../src/screens/board/tabs/builds/model';
import { launcherDoc } from './fixtures';
import { input, launchers } from './model-docs';

const views = (...docs: object[]) => launcherViews(input({ launchers: launchers(...docs) }));

const names = (list: { launcher: { name: string } }[]): string[] => list.map((view) => view.launcher.name);

describe('I-105 builds filter, sort and group', { tags: ['critical'] }, () => {
  it('filters by search over name and branch, and by item and kind', () => {
    const all = views(launcherDoc('web', { for: 'I-1', branch: 'feat/x', kind: 'app' }), launcherDoc('api', { for: 'I-2', kind: 'svc' }));
    const spec = { order: 'name', direction: 'asc' } as const;
    expect(names(buildsView(all, { ...spec, filters: { ...BUILD_FILTERS, query: 'feat' } }).live)).toEqual(['web']);
    expect(names(buildsView(all, { ...spec, filters: { ...BUILD_FILTERS, item: ['I-2'] } }).live)).toEqual(['api']);
    expect(names(buildsView(all, { ...spec, filters: { ...BUILD_FILTERS, kind: ['app'] } }).live)).toEqual(['web']);
  });

  it('keeps the master first whatever the order, then sorts by name', () => {
    const all = views(launcherDoc('zeta'), launcherDoc('master', { master: true }), launcherDoc('alpha'));
    const result = buildsView(all, { filters: BUILD_FILTERS, order: 'name', direction: 'desc' });
    expect(names(result.live)[0]).toBe('master');
    expect(names(result.live).slice(1)).toEqual(['zeta', 'alpha']);
  });

  it('groups by kind with the unkinded last', () => {
    const all = views(launcherDoc('a', { kind: 'svc' }), launcherDoc('b', { kind: undefined }), launcherDoc('c', { kind: 'app' }));
    expect(groupByKind(all.live).map((group) => group.label)).toEqual(['app', 'svc', 'No kind']);
  });
});
