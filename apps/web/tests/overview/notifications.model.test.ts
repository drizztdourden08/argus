/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { NoteItem } from '../../src/screens/overview/notifications/model';
import {
  apiSentence, countLevels, filterNotes, noteLevel, noteProject, projectCounts, settingTip, sortNotes, testSentence, toastNote,
} from '../../src/screens/overview/notifications/model';
import { ApiError } from '../../src/data/api';

const note = (id: string, extra: Partial<NoteItem> = {}): NoteItem => ({ id, n: 1, at: '2026-10-26T11:00:00.000Z', level: 'ping', title: 'T', text: 'x', ...extra });

describe('the record and its order', () => {
  it('runs newest first by the number, then by the time', () => {
    const rows = [note('n-1', { n: 1 }), note('n-3', { n: 3 }), note('n-2', { n: 2 })];
    expect(sortNotes(rows).map((row) => row.id)).toEqual(['n-3', 'n-2', 'n-1']);
    const noNumber = [note('a', { n: undefined, at: '2026-10-26T10:00:00.000Z' }), note('b', { n: undefined, at: '2026-10-26T12:00:00.000Z' })];
    expect(sortNotes(noNumber).map((row) => row.id)).toEqual(['b', 'a']);
  });

  it('reads an unknown level, or none, as a ping, and a missing project as empty', () => {
    expect(noteLevel(note('a', { level: 'urgent' }))).toBe('urgent');
    expect(noteLevel(note('a', { level: 'loud' }))).toBe('ping');
    expect(noteLevel({ id: 'a' })).toBe('ping');
    expect(noteProject(note('a', { project: 'rotp' }))).toBe('rotp');
    expect(noteProject(note('a', { project: null }))).toBe('');
  });

  it('counts the levels and the projects, projects sorted by key', () => {
    const rows = [note('1', { level: 'info', project: 'rotp' }), note('2', { project: 'alpha' }), note('3', { level: 'urgent', project: 'rotp' }), note('4', { project: null })];
    expect(countLevels(rows)).toEqual({ all: 4, info: 1, ping: 2, urgent: 1 });
    expect(projectCounts(rows)).toEqual([['alpha', 1], ['rotp', 2]]);
  });

  it('filters by level and by project, and both combine; all clears', () => {
    const rows = [note('1', { level: 'info', project: 'rotp' }), note('2', { level: 'ping', project: 'alpha' }), note('3', { level: 'ping', project: 'rotp' })];
    expect(filterNotes(rows, { level: 'all', project: '' })).toHaveLength(3);
    expect(filterNotes(rows, { level: 'ping', project: '' }).map((row) => row.id)).toEqual(['2', '3']);
    expect(filterNotes(rows, { level: 'all', project: 'rotp' }).map((row) => row.id)).toEqual(['1', '3']);
    expect(filterNotes(rows, { level: 'ping', project: 'rotp' }).map((row) => row.id)).toEqual(['3']);
    expect(filterNotes(rows, { level: 'urgent', project: '' })).toEqual([]);
  });
});

describe('what became of the toast', () => {
  it('says shown, with the reason when there is one', () => {
    expect(toastNote(note('a', { toast: 'shown' }))).toEqual({ text: 'toast shown', bad: false });
    expect(toastNote(note('a', { toast: 'shown', why: 'shown as a tray balloon (no PowerShell)' })).text).toBe('toast shown (shown as a tray balloon (no PowerShell))');
  });

  it('says failed in the bad tone, with the reason', () => {
    expect(toastNote(note('a', { toast: 'failed', why: 'PowerShell exit 2' }))).toEqual({ text: 'toast failed: PowerShell exit 2', bad: true });
    expect(toastNote(note('a', { toast: 'failed' })).text).toBe('toast failed');
  });

  it('says limited with its reason, in the plain tone', () => {
    expect(toastNote(note('a', { toast: 'limited', why: 'you pinged 4 times in a minute' }))).toEqual({ text: 'not shown (limited: you pinged 4 times in a minute)', bad: false });
  });

  it('says info is recorded only, disabled when notify.os is off, and the reason or skipped otherwise', () => {
    expect(toastNote(note('a', { level: 'info', toast: 'skipped' })).text).toBe('not shown (info is recorded only)');
    expect(toastNote(note('a', { toast: 'skipped', why: 'OS notifications are off (config notify.os)' })).text).toBe('not shown (disabled)');
    expect(toastNote(note('a', { toast: 'skipped', why: 'ping is a quiet level' })).text).toBe('not shown (ping is a quiet level)');
    expect(toastNote(note('a', { toast: 'skipped' })).text).toBe('not shown (skipped)');
  });
});

describe('the sentences of the test button and the settings', () => {
  it('says the toast should be on screen, or why there was none', () => {
    expect(testSentence({ toast: 'shown' })).toBe('Sent: the toast should be on your screen now.');
    expect(testSentence({ toast: 'limited', why: 'you pinged 4 times in a minute' })).toBe('No toast: you pinged 4 times in a minute. It is recorded below.');
    expect(testSentence({ toast: 'failed' })).toBe('No toast: failed. It is recorded below.');
    expect(testSentence({})).toBe('No toast: unknown reason. It is recorded below.');
  });

  it('says the dashboard is unreachable, that only the page may change the settings, or the message', () => {
    const otherwise = (detail: string): string => `Not saved: ${detail}.`;
    expect(apiSentence(new ApiError(0, { code: 'unavailable', message: 'down' }), otherwise)).toBe('Dashboard unreachable, retrying.');
    expect(apiSentence(new ApiError(403, { code: 'forbidden', message: 'no' }), otherwise)).toBe('Only this page may change the settings.');
    expect(apiSentence(new ApiError(500, { code: 'config_not_saved', message: 'disk full' }), otherwise)).toBe('Not saved: disk full.');
    expect(apiSentence('odd', otherwise)).toBe('Not saved: unknown.');
  });
});

describe('the tip of a setting', () => {
  it('says what a toast switch does in each state, and that info is recorded only by default', () => {
    expect(settingTip('toast', 'urgent', true)).toBe('A toast is shown for urgent');
    expect(settingTip('toast', 'ping', false)).toBe('No toast for ping');
    expect(settingTip('toast', 'info', false)).toBe('No toast for info (info is recorded only by default)');
  });

  it('says whether the toast of a level plays the system sound', () => {
    expect(settingTip('sound', 'urgent', true)).toBe('The urgent toast plays the system notification sound');
    expect(settingTip('sound', 'ping', false)).toBe('The ping toast is silent');
  });
});
