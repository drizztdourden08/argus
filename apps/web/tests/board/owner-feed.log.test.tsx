/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { NOW, byId, freeText, notice } from '../helpers/board-docs';
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

describe('the view of the activity', () => {
  it('shows what waits on you, the notices and the work log under All', async () => {
    await openBoard({ entries: byId(freeText, notice), log: logOf(2) });
    expect(screen.getByText('Waiting on you · 1')).toBeTruthy();
    expect(screen.getByText('Notices · 1')).toBeTruthy();
    expect(screen.getByText('Work log')).toBeTruthy();
  });

  it('Needs you leaves out the notices and the log, Log only the log, and the choice is remembered', async () => {
    await openBoard({ entries: byId(freeText, notice), log: logOf(2) });
    fireEvent.click(screen.getByRole('radio', { name: 'Needs you' }));
    expect(screen.getByText('Waiting on you · 1')).toBeTruthy();
    expect(screen.queryByText('Notices · 1')).toBeNull();
    expect(screen.queryByText('Work log')).toBeNull();
    fireEvent.click(screen.getByRole('radio', { name: 'Log' }));
    expect(screen.queryByText('Waiting on you · 1')).toBeNull();
    expect(screen.getByText('Work log')).toBeTruthy();
    expect(window.localStorage.getItem('coord.feed')).toBe('log');
    cleanup();
    await openBoard({ entries: byId(freeText, notice), log: logOf(2) });
    expect(screen.getByRole('radio', { name: 'Log' }).getAttribute('aria-checked')).toBe('true');
  });

  it('says what is missing for each view', async () => {
    await openBoard({});
    expect(screen.getByText('Nothing here yet.')).toBeTruthy();
    fireEvent.click(screen.getByRole('radio', { name: 'Needs you' }));
    expect(screen.getByText('Nothing waiting on you.')).toBeTruthy();
    fireEvent.click(screen.getByRole('radio', { name: 'Log' }));
    expect(screen.getByText('Nothing logged yet.')).toBeTruthy();
  });
});

describe('the work log', () => {
  it('lists the newest lines first with the thread of each', async () => {
    await openBoard({ log: logOf(3) });
    const items = logItems();
    expect(items).toHaveLength(3);
    expect(items[0]?.textContent).toContain('line 3');
    expect(items[2]?.textContent).toContain('line 1');
    expect(within(items[0] as HTMLElement).getByText('lead')).toBeTruthy();
  });

  it('shows twelve lines, then 25 more at a time, and Show less goes back', async () => {
    await openBoard({ log: logOf(40) });
    expect(logItems()).toHaveLength(12);
    fireEvent.click(screen.getByRole('button', { name: 'Show 25 more (28 left)' }));
    expect(logItems()).toHaveLength(37);
    fireEvent.click(screen.getByRole('button', { name: 'Show 3 more (3 left)' }));
    expect(logItems()).toHaveLength(40);
    expect(screen.queryByRole('button', { name: /more/ })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show less' }));
    expect(logItems()).toHaveLength(12);
  });

  it('a line keeps plain text: the marks are removed, the whole text is in the title', async () => {
    const marked = { seq: 1, ts: NOW, thread: 'lead', item: 'I-2', text: 'did **the** `thing`', subject: 'audit' };
    await openBoard({ log: { audit: { subject: 'audit', entries: [marked] } } });
    const text = screen.getByText('did the thing');
    expect(text.getAttribute('title')).toBe('did **the** `thing`');
    expect(screen.getByText('I-2')).toBeTruthy();
    expect(screen.getByText('audit')).toBeTruthy();
  });
});

describe('the tests waiting on you', () => {
  const manual = (id: string, over: object = {}) => ({
    id,
    title: `${id} title`,
    thread: 'coordinator',
    mode: 'manual',
    status: 'pending',
    updatedAt: NOW,
    ...over,
  });

  it('the tests of one bundle are ONE card with a count', async () => {
    const tests = ['T-1', 'T-2', 'T-3'].map((id, index) => manual(id, { build: 'bundle-4', rank: index + 1 }));
    await openBoard({ tests: byId(...tests) });
    expect(screen.getAllByRole('button', { name: 'Open' })).toHaveLength(1);
    expect(screen.getByText('3 tests to run')).toBeTruthy();
    expect(screen.getByText(/T-1 title \.\.\./)).toBeTruthy();
    expect(screen.getAllByText('bundle-4')).toHaveLength(1);
  });

  it('a test with no bundle is its own line with an Open button for the test', async () => {
    await openBoard({ tests: byId(manual('T-7')) });
    expect(screen.getByText('T-7 title')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Open' })).toBeTruthy();
  });

  it('a closed bundle leaves the feed', async () => {
    await openBoard({ tests: byId(manual('T-1', { build: 'old' })), builds: { old: { id: 'old', closedAt: NOW } } });
    expect(screen.queryByText('T-1 title')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Open' })).toBeNull();
  });
});

describe('the proof images of a test', () => {
  const proofs = [{ n: 1, url: '/files/rotp/proofs/T-9-1.png', caption: 'first shot' }];
  const shotTest = { id: 'T-9', title: 'Shot test', thread: 'coordinator', mode: 'manual', status: 'pending', updatedAt: NOW, proofs };

  it('are not in the waiting row, which shows the title only as Node does; the test drawer holds them', async () => {
    await openBoard({ tests: byId(shotTest) });
    expect(screen.getByText('Shot test')).toBeTruthy();
    expect(screen.queryByRole('group', { name: 'Proofs of Shot test' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Open first shot' })).toBeNull();
  });
});
