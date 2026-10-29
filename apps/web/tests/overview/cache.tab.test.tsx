/* @layer renderer-app @kind test */
import { formatDate } from '@drizztdourden08/tessera';
import { cleanup, fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { CacheReport } from '../../src/data/cache';
import { renderAt } from '../helpers/render-at';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';
import type { ApiCall } from './helpers/stub-api';

installHarness();

const MB = 1024 * 1024;
const DAY = 86_400_000;
const CORE = '@drizztdourden08/brock-core';
const LATER = new Date(Date.now() + 5 * DAY).toISOString();

const ago = (days: number): string => new Date(Date.now() - days * DAY).toISOString();

const REPORT: CacheReport = {
  root: 'C:/Users/me/.brock/cache/build-cache',
  exists: true,
  graceDays: 14,
  shadow: false,
  problems: [],
  lastPrune: null,
  totals: {
    versions: 3,
    entries: 4,
    sizeBytes: 9 * MB,
    inUseBytes: 4 * MB,
    unusedBytes: 5 * MB,
    prunableEntries: 1,
    prunableBytes: 3 * MB,
    abandonedFolders: 1,
    abandonedBytes: MB,
  },
  rows: [
    { kind: 'package', name: CORE, version: '0.115.3', buildKeys: 2, users: [{ label: 'brock', kind: 'project' }, { label: 'brock / agent-1', kind: 'worktree' }], userCount: 2, sizeBytes: 4 * MB, lastUsedAt: ago(2), inUse: true, deleteAfter: null, prunable: 0, prunableBytes: 0 },
    { kind: 'electron', name: 'electron', version: '41.0.0', buildKeys: 1, users: [], userCount: 0, sizeBytes: 3 * MB, lastUsedAt: ago(30), inUse: false, deleteAfter: ago(16), prunable: 1, prunableBytes: 3 * MB },
    { kind: 'package', name: '@drizztdourden08/tessera', version: '0.80.0', buildKeys: 1, users: [], userCount: 0, sizeBytes: 2 * MB, lastUsedAt: ago(9), inUse: false, deleteAfter: LATER, prunable: 0, prunableBytes: 0 },
  ],
};

const PRUNED = { at: new Date().toISOString(), deleted: [{ kind: 'electron', name: 'electron', version: '41.0.0', buildKey: 'win32-x64', sizeBytes: 3 * MB }], abandonedDeleted: 1, freedBytes: 4 * MB, refused: [], failed: [] };

const serve = (report: CacheReport = REPORT, prune: { status?: number; body?: unknown } = { body: PRUNED }): ApiCall[] =>
  stubApi((call) => {
    if (call.path === '/api/cache') return { body: report };
    if (call.path === '/api/cache/prune') return prune;
    return undefined;
  });

const open = async (report: CacheReport = REPORT, prune?: { status?: number; body?: unknown }) => {
  const calls = serve(report, prune);
  renderAt('/#cache');
  await screen.findAllByText(CORE);
  return calls;
};

const shown = (text: string | RegExp): number => screen.getAllByText(text).length;

const reads = (calls: ApiCall[]): number => calls.filter((call) => call.method === 'GET' && call.path === '/api/cache').length;

describe('the Cache tab listing', () => {
  it('has a tab beside the other overview tabs', async () => {
    serve();
    renderAt('/#cache');
    expect(await screen.findByRole('tab', { name: /Cache/ })).toBeTruthy();
    expect(screen.getByRole('tab', { name: /Processes/ })).toBeTruthy();
  });

  it('shows a row per version with its kind, size, build count and users', async () => {
    await open();
    expect(shown('Package')).toBe(2);
    expect(shown('Electron')).toBe(1);
    expect(shown('0.115.3')).toBeGreaterThan(0);
    expect(shown('brock, brock / agent-1')).toBeGreaterThan(0);
    expect(shown('4.0 MB')).toBeGreaterThan(0);
    expect(shown('@drizztdourden08/tessera')).toBeGreaterThan(0);
  });

  it('says when an unused version goes: at the next prune once past the grace period, else after a date', async () => {
    await open();
    expect(shown('unused, deleted at the next prune')).toBeGreaterThan(0);
    expect(shown(`unused, deleted after ${formatDate(Date.parse(LATER), 'date')}`)).toBeGreaterThan(0);
  });

  it('shows the totals, the folder and the grace period', async () => {
    await open();
    expect(screen.getByText('3 versions')).toBeTruthy();
    expect(screen.getByText('9.0 MB total')).toBeTruthy();
    expect(screen.getByText('4.0 MB in use')).toBeTruthy();
    expect(screen.getByText('5.0 MB unused')).toBeTruthy();
    expect(screen.getByText('3.0 MB ready to delete')).toBeTruthy();
    expect(screen.getByText('not pruned since Argus started')).toBeTruthy();
    expect(screen.getByText(/build-cache\. An unused entry is deleted 14 days after its last use\./)).toBeTruthy();
  });

  it('says there is no folder yet and when the cache is empty', async () => {
    serve({ ...REPORT, exists: false, rows: [] });
    renderAt('/#cache');
    expect(await screen.findByText(/no build cache folder yet/)).toBeTruthy();
    cleanup();
    serve({ ...REPORT, rows: [] });
    renderAt('/#cache');
    expect(await screen.findByText('The build cache is empty.')).toBeTruthy();
  });

  it('says why when the read fails', async () => {
    stubApi(() => ({ status: 503, body: { error: { code: 'unavailable', message: 'the build cache service is not running' } } }));
    renderAt('/#cache');
    expect(await screen.findByText('The build cache could not be read: the build cache service is not running.')).toBeTruthy();
  });

  it('warns when a lockfile could not be read, and in shadow mode', async () => {
    await open({ ...REPORT, shadow: true, problems: ['X:/a/pnpm-lock.yaml: denied'] });
    expect(screen.getByText(/nothing is deleted until it can: X:\/a\/pnpm-lock\.yaml: denied/)).toBeTruthy();
    expect(screen.getByText(/shadow mode/)).toBeTruthy();
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Prune now' }).disabled).toBe(true);
  });
});

describe('Prune now', () => {
  it('lists what will go before it deletes, and Cancel deletes nothing', async () => {
    const calls = await open();
    fireEvent.click(screen.getByRole('button', { name: 'Prune now' }));
    expect(await screen.findByText('Delete 1 unused entry (3.0 MB) that passed 14 days without use?')).toBeTruthy();
    expect(screen.getByText('Electron electron 41.0.0: 1 build, 3.0 MB')).toBeTruthy();
    expect(screen.getByText(/Also 1 left-over temp folder \(1\.0 MB\)/)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByText(/that passed 14 days/)).toBeNull());
    expect(calls.filter((call) => call.method === 'POST')).toEqual([]);
  });

  it('deletes on confirm, says what it freed and reads the cache again', async () => {
    const calls = await open();
    fireEvent.click(screen.getByRole('button', { name: 'Prune now' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('Deleted 1 cache entry and 1 temp folder, 4.0 MB freed.')).toBeTruthy();
    await waitFor(() => expect(reads(calls)).toBe(2));
    expect(calls.filter((call) => call.method === 'POST').map((call) => call.path)).toEqual(['/api/cache/prune']);
  });

  it('says why nothing was deleted when the server refuses', async () => {
    await open(REPORT, { status: 409, body: { error: { code: 'conflict', message: 'the worktrees were not scanned yet' } } });
    fireEvent.click(screen.getByRole('button', { name: 'Prune now' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('Nothing was deleted: the worktrees were not scanned yet.')).toBeTruthy();
  });

  it('is off, with the reason, when nothing is past its grace period', async () => {
    await open({ ...REPORT, totals: { ...REPORT.totals, prunableEntries: 0, prunableBytes: 0, abandonedFolders: 0, abandonedBytes: 0 } });
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Prune now' }).disabled).toBe(true);
    expect(screen.getByText('Nothing is past its grace period, so there is nothing to delete.')).toBeTruthy();
  });
});
