/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { formatDuration, shortCommit } from '../../src/format';
import { sortPrs, stepStatus } from '../../src/screens/board/tabs/changes/model';
import { sortVersions } from '../../src/screens/releases/model';
import { ago, MIN, prDoc, stepDoc, versionDoc } from './helpers/fixtures';

describe('the duration of a step', () => {
  it('reads in milliseconds, seconds and minutes', () => {
    expect(formatDuration(250)).toBe('250ms');
    expect(formatDuration(1500)).toBe('1.5s');
    expect(formatDuration(42_000)).toBe('42s');
    expect(formatDuration(65_000)).toBe('1m 5s');
    expect(formatDuration(120_000)).toBe('2m');
    expect(formatDuration(null)).toBe('-');
  });

  it('cuts a commit to seven characters', () => {
    expect(shortCommit('0123456789abcdef')).toBe('0123456');
    expect(shortCommit(null)).toBe('-');
  });
});

describe('the state of a step', () => {
  it('is its status when it has one', () => {
    expect(stepStatus({ ...stepDoc('a'), status: 'skipped-local' })).toBe('skipped-local');
  });

  it('comes from the exit code when it has none', () => {
    expect(stepStatus(stepDoc('a'))).toBe('pass');
    expect(stepStatus(stepDoc('a', { exitCode: 3 }))).toBe('fail');
    expect(stepStatus(stepDoc('a', { exitCode: null }))).toBe('queued');
  });
});

describe('the order of the lists', () => {
  it('puts open pull requests first, then the newest', () => {
    const docs = [prDoc('PR-1', { status: 'merged', createdAt: ago(MIN) }), prDoc('PR-2', { createdAt: ago(9 * MIN) }), prDoc('PR-3', { createdAt: ago(2 * MIN) })];
    expect(sortPrs(docs).map((pr) => pr.id)).toEqual(['PR-3', 'PR-2', 'PR-1']);
  });

  it('puts the newest version first', () => {
    const docs = [versionDoc('1.0.0', { publishedAt: ago(9 * MIN) }), versionDoc('1.1.0', { publishedAt: ago(MIN) })];
    expect(sortVersions(docs).map((version) => version.version)).toEqual(['1.1.0', '1.0.0']);
  });
});
