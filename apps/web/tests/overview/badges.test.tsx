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

describe('the tabs and their badges', () => {
  it('has the four tabs, Dashboards first, and no badge on a tab with nothing to count', async () => {
    await open();
    for (const label of ['Dashboards', 'Processes', 'Activity', 'Notifications']) expect(tab(label)).toBeTruthy();
    expect(tab('Dashboards').getAttribute('aria-selected')).toBe('true');
    expect(tab('Dashboards').textContent).toBe('Dashboards');
  });

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
    expect(tab('Activity').textContent).toBe('Activity');
    expect(localStorage.getItem('argus.activity.seen')).not.toBeNull();
  });

  it('keeps only what arrives later as unread after the tab was read', async () => {
    const view = await open('/#activity');
    await waitFor(() => expect(localStorage.getItem('argus.activity.seen')).not.toBeNull());
    fireEvent.click(tab('Dashboards'));
    await waitFor(() => expect(tab('Dashboards').getAttribute('aria-selected')).toBe('true'));
    expect(tab('Activity').textContent).toBe('Activity');
    await feed(view.sources, {
      activity: { ...ACTIVITY, 'a-9': { n: 9, at: new Date(Date.now() + 60000).toISOString(), kind: 'fault', level: 'warn', text: 'later' } },
    });
    await waitFor(() => expect(tab('Activity').textContent).toContain('1'));
  });

  it('starts with no unread on Activity when the page opens on that tab', async () => {
    await open('/#activity');
    expect(tab('Activity').getAttribute('aria-selected')).toBe('true');
    expect(tab('Activity').textContent).toBe('Activity');
  });

  it('keeps an older entry read after a reload', async () => {
    localStorage.setItem('argus.activity.seen', minutesAgo(0));
    await open();
    expect(tab('Activity').textContent).toBe('Activity');
  });

  it('opens the tab named by the hash, and falls back on Dashboards for a hash it does not know', async () => {
    await open('/#notifications');
    expect(tab('Notifications').getAttribute('aria-selected')).toBe('true');
    cleanup();
    await open('/#nowhere');
    expect(tab('Dashboards').getAttribute('aria-selected')).toBe('true');
  });
});

const tone = (name: string, color: string): boolean => tab(name).querySelector(`.badge--${color}`) !== null;

describe('the tone of the tab badges', () => {
  it('tints Processes warning, and Activity and Notifications danger while an error or an urgent ping is unread', async () => {
    await open();
    await waitFor(() => expect(tab('Processes').textContent).toContain('2'));
    expect(tone('Processes', 'warning')).toBe(true);
    expect(tone('Activity', 'danger')).toBe(true);
    expect(tone('Notifications', 'danger')).toBe(true);
  });

  it('tints Activity and Notifications warning when the unread ones are neither an error nor urgent', async () => {
    const view = renderAt('/');
    const activity = { 'a-3': ACTIVITY['a-3'] };
    const notifications = { 'n-1': { ...NOTIFICATIONS['n-1'], at: minutesAgo(0) } };
    await feed(view.sources, { cards: CARDS, procs: PROCS, meta: PROCS_META, activity, notifications });
    await waitFor(() => expect(tab('Activity').textContent).toContain('1'));
    expect(tone('Activity', 'warning')).toBe(true);
    expect(tone('Notifications', 'warning')).toBe(true);
    expect(tone('Activity', 'danger')).toBe(false);
  });
});
