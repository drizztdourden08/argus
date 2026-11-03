/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { byId, freeText, notice } from '../helpers/board-docs';
import { openBoard, resetBoard } from '../helpers/open-board';
import { mockPosts } from '../helpers/post-mock';

beforeEach(() => {
  window.localStorage.clear();
  mockPosts();
});

afterEach(resetBoard);

const line = (n: number) => ({ seq: n, ts: `2026-10-26T08:${String(n).padStart(2, '0')}:00.000Z`, thread: 'lead', item: null, text: `line ${n}` });

const logOf = (count: number) => ({ audit: { subject: 'audit', entries: Array.from({ length: count }, (_, index) => line(index + 1)) } });

const logItems = () => within(screen.getByRole('list', { name: 'Work log' })).getAllByRole('listitem');

describe('S1 the view of the activity', { tags: ['standard'] }, () => {
  it('Needs you leaves out the notices and the log, Log only the log, and the choice is remembered', async () => {
    await openBoard({ entries: byId(freeText, notice), log: logOf(2) });
    expect(screen.getByText('Log · 3')).toBeTruthy();
    fireEvent.click(screen.getByRole('radio', { name: 'Needs you' }));
    expect(screen.getByText('Waiting on you · 1')).toBeTruthy();
    expect(screen.queryByText('Log · 3')).toBeNull();
    expect(screen.queryByText('Work log')).toBeNull();
    fireEvent.click(screen.getByRole('radio', { name: 'Log' }));
    expect(screen.queryByText('Waiting on you · 1')).toBeNull();
    expect(screen.getByText('Work log')).toBeTruthy();
    expect(window.localStorage.getItem('coord.feed')).toBe('log');
    cleanup();
    await openBoard({ entries: byId(freeText, notice), log: logOf(2) });
    expect(screen.getByRole('radio', { name: 'Log' }).getAttribute('aria-checked')).toBe('true');
  });
});

describe('S1 the work log', { tags: ['standard'] }, () => {
  it('shows twelve lines, then 25 more at a time, and Show less goes back', async () => {
    await openBoard({ log: logOf(40) });
    fireEvent.click(screen.getByText('Log · 40'));
    expect(logItems()).toHaveLength(12);
    fireEvent.click(screen.getByRole('button', { name: 'Show 25 more (28 left)' }));
    expect(logItems()).toHaveLength(37);
    fireEvent.click(screen.getByRole('button', { name: 'Show 3 more (3 left)' }));
    expect(logItems()).toHaveLength(40);
    expect(screen.queryByRole('button', { name: /more/ })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show less' }));
    expect(logItems()).toHaveLength(12);
  });
});
