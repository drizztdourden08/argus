/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { ACTIVITY, CARDS, minutesAgo, NOTIFICATIONS, PROCS, PROCS_META } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';

installHarness();
beforeEach(() => {
  stubApi();
});

const tab = (name: string): HTMLElement => screen.getByRole('tab', { name: new RegExp(name) });

const open = async (location = '/') => {
  const view = renderAt(location);
  await feed(view.sources, { cards: CARDS, procs: PROCS, meta: PROCS_META, activity: ACTIVITY, notifications: NOTIFICATIONS });
  return view;
};

describe('S1 the tabs and their badges', { tags: ['standard'] }, () => {
  it('counts the processes that need a look on Processes', async () => {
    await open();
    await waitFor(() => expect(tab('Processes').textContent).toContain('2'));
  });

  it('counts the warnings and errors of the feed that were never read on Activity, never the info', async () => {
    await open();
    await waitFor(() => expect(tab('Activity').textContent).toContain('2'));
  });

  it('counts the notifications that were never read on Notifications', async () => {
    await open();
    await waitFor(() => expect(tab('Notifications').textContent).toContain('2'));
  });

  it('reads the Activity badge when the tab opens and remembers the time', async () => {
    await open();
    await waitFor(() => expect(tab('Activity').textContent).toContain('2'));
    fireEvent.click(tab('Activity'));
    await waitFor(() => expect(tab('Activity').getAttribute('aria-selected')).toBe('true'));
    expect(tab('Activity').textContent).toBe('Activity0');
    expect(localStorage.getItem('argus.activity.seen')).not.toBeNull();
  });

  it('keeps only what arrives later as unread after the tab was read', async () => {
    const view = await open('/#activity');
    await waitFor(() => expect(localStorage.getItem('argus.activity.seen')).not.toBeNull());
    fireEvent.click(tab('Dashboards'));
    await waitFor(() => expect(tab('Dashboards').getAttribute('aria-selected')).toBe('true'));
    expect(tab('Activity').textContent).toBe('Activity0');
    await feed(view.sources, {
      activity: { ...ACTIVITY, 'a-9': { n: 9, at: '2999-01-01T00:00:00.000Z', kind: 'fault', level: 'warn', text: 'later' } },
    });
    await waitFor(() => expect(tab('Activity').textContent).toContain('1'));
  });

  it('keeps an older entry read after a reload', async () => {
    localStorage.setItem('argus.activity.seen', minutesAgo(0));
    await open();
    expect(tab('Activity').textContent).toBe('Activity0');
  });

  it('opens the tab named by the hash, and falls back on Dashboards for a hash it does not know', async () => {
    await open('/#notifications');
    expect(tab('Notifications').getAttribute('aria-selected')).toBe('true');
    cleanup();
    await open('/#nowhere');
    expect(tab('Dashboards').getAttribute('aria-selected')).toBe('true');
  });
});
