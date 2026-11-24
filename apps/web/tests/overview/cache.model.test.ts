/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { formatBytes } from '../../src/format';
import { ApiError } from '../../src/data/api';
import { cacheError, pruneSaid } from '../../src/screens/overview/packages/cache/model';

const EMPTY = { at: '2026-10-29T12:00:00.000Z', deleted: [], abandonedDeleted: 0, freedBytes: 0, refused: [], failed: [] };

describe('K1 formatBytes', { tags: ['critical'] }, () => {
  it('uses the unit that keeps a number small and drops the decimal from a hundred', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(300)).toBe('300 B');
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(4 * 1024 * 1024)).toBe('4.0 MB');
    expect(formatBytes(250 * 1024 * 1024)).toBe('250 MB');
    expect(formatBytes(3 * 1024 ** 3)).toBe('3.0 GB');
  });

  it('shows a dash for what is not a number', () => {
    expect(formatBytes(null)).toBe('-');
    expect(formatBytes('12')).toBe('-');
  });
});

describe('K4 the prune sentences', { tags: ['critical'] }, () => {
  it('counts what a prune deleted and what it freed', () => {
    const said = pruneSaid({ ...EMPTY, abandonedDeleted: 2, freedBytes: 2048, deleted: [{ kind: 'package', name: 'n', version: '1.0.0', buildKey: 'node-aa', sizeBytes: 2048 }] });
    expect(said).toEqual({ variant: 'success', message: 'Deleted 1 cache entry and 2 temp folders, 2.0 KB freed.' });
  });

  it('turns red when a delete was refused or failed', () => {
    const said = pruneSaid({ ...EMPTY, refused: ['x is outside the cache root'], failed: ['y: busy'] });
    expect(said.variant).toBe('danger');
    expect(said.message).toContain('2 deletes were refused or failed');
  });

  it('explains a refusal, shadow mode and an unreachable dashboard', () => {
    const otherwise = (detail: string): string => `failed: ${detail}`;
    expect(cacheError(new ApiError(403, { code: 'read_only', message: 'shadow' }), otherwise)).toContain('shadow mode');
    expect(cacheError(new ApiError(409, { code: 'conflict', message: 'not scanned' }), otherwise)).toBe('Nothing was deleted: not scanned.');
    expect(cacheError(new ApiError(0, { code: 'unreachable', message: 'down' }), otherwise)).toBe('Dashboard unreachable, retrying.');
    expect(cacheError(new Error('boom'), otherwise)).toBe('failed: boom');
  });
});
