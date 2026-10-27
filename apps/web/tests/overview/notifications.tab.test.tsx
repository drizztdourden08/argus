/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { CARDS, NOTIFICATIONS } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';
import type { ApiCall } from './helpers/stub-api';

const SETTINGS = { os: true, toast: { info: false, ping: true, urgent: true }, sound: { info: false, ping: false, urgent: true } };

installHarness();

const open = async (notifications: Record<string, object> = NOTIFICATIONS) => {
  const view = renderAt('/#notifications');
  await feed(view.sources, { cards: CARDS, notifications });
  return view;
};

const posts = (calls: ApiCall[], path: string): ApiCall[] => calls.filter((call) => call.method === 'POST' && call.path === path);

describe('the Notifications tab record', () => {
  it('says there is none yet, with the plain sentence', async () => {
    stubApi((call) => (call.path === '/api/notify/settings' ? { body: SETTINGS } : undefined));
    await open({});
    expect(await screen.findByText('No notification yet. Threads ping you here when you are needed.')).toBeTruthy();
  });

  it('shows the title, the text, the level, the project link and the note on the toast', async () => {
    stubApi((call) => (call.path === '/api/notify/settings' ? { body: SETTINGS } : undefined));
    await open();
    expect(await screen.findByText('RotP: build ready')).toBeTruthy();
    expect(screen.getByText('Build is ready to test')).toBeTruthy();
    expect(screen.getByText('toast shown')).toBeTruthy();
    expect(screen.getByText('not shown (info is recorded only)')).toBeTruthy();
    expect(screen.getByRole('link', { name: 'RotP' }).getAttribute('href')).toBe('/p/rotp/');
  });

  it('filters by level with the counts on the buttons, and the all button clears it', async () => {
    stubApi((call) => (call.path === '/api/notify/settings' ? { body: SETTINGS } : undefined));
    await open();
    await screen.findByText('RotP: build ready');
    expect(screen.getByText('All levels 2')).toBeTruthy();
    fireEvent.click(screen.getByText('urgent 1'));
    await waitFor(() => expect(screen.queryByText('FYI')).toBeNull());
    expect(screen.getByText('RotP: build ready')).toBeTruthy();
    fireEvent.click(screen.getByText('All levels 2'));
    expect(await screen.findByText('FYI')).toBeTruthy();
  });

  it('filters by project with a chip per project and its count, and the all chip clears it', async () => {
    stubApi((call) => (call.path === '/api/notify/settings' ? { body: SETTINGS } : undefined));
    await open();
    await screen.findByText('RotP: build ready');
    expect(screen.getByRole('radio', { name: 'All projects 2' }).getAttribute('aria-checked')).toBe('true');
    fireEvent.click(screen.getByRole('radio', { name: 'RotP 1' }));
    await waitFor(() => expect(screen.queryByText('FYI')).toBeNull());
    expect(screen.getByText('RotP: build ready')).toBeTruthy();
    fireEvent.click(screen.getByRole('radio', { name: 'RotP 1' }));
    expect(await screen.findByText('FYI')).toBeTruthy();
  });

  it('says nothing matches when a level has no entries', async () => {
    stubApi((call) => (call.path === '/api/notify/settings' ? { body: SETTINGS } : undefined));
    await open();
    await screen.findByText('RotP: build ready');
    fireEvent.click(screen.getByText('ping 0'));
    expect(await screen.findByText('Nothing matches these filters.')).toBeTruthy();
  });

  it('shows a note with no time, no level and no number as a row', async () => {
    stubApi((call) => (call.path === '/api/notify/settings' ? { body: SETTINGS } : undefined));
    await open({ 'n-5': { text: 'bare' } });
    expect(await screen.findByText('bare')).toBeTruthy();
  });
});

describe('the test button', () => {
  it('posts to /api/notify/test and says the toast should be on screen', async () => {
    const calls = stubApi((call) => (call.path === '/api/notify/test' ? { body: { toast: 'shown' } } : { body: SETTINGS }));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send a test notification' }));
    expect(await screen.findByText('Sent: the toast should be on your screen now.')).toBeTruthy();
    expect(posts(calls, '/api/notify/test')).toHaveLength(1);
  });

  it('says why there was no toast, and that it is recorded', async () => {
    stubApi((call) => (call.path === '/api/notify/test' ? { body: { toast: 'limited', why: 'you pinged 4 times in a minute' } } : { body: SETTINGS }));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send a test notification' }));
    expect(await screen.findByText('No toast: you pinged 4 times in a minute. It is recorded below.')).toBeTruthy();
  });

  it('says not sent with the message when the route fails', async () => {
    stubApi((call) => (call.path === '/api/notify/test' ? { status: 500, body: { error: { code: 'internal', message: 'boom' } } } : { body: SETTINGS }));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send a test notification' }));
    expect(await screen.findByText('Not sent: boom.')).toBeTruthy();
  });
});

describe('the settings row', () => {
  it('reads the settings when the tab opens, not before', async () => {
    const calls = stubApi(() => ({ body: SETTINGS }));
    const view = renderAt('/');
    await feed(view.sources, { cards: CARDS });
    expect(calls.filter((call) => call.path === '/api/notify/settings')).toEqual([]);
    cleanup();
    await open();
    await screen.findByText('Notification settings');
    expect(calls.filter((call) => call.path === '/api/notify/settings')).toHaveLength(1);
  });

  it('shows a Toast and a Sound toggle per level', async () => {
    stubApi(() => ({ body: SETTINGS }));
    await open();
    const ping = await screen.findByRole('group', { name: 'ping' });
    expect(within(ping).getByRole('button', { name: 'Toast' })).toBeTruthy();
    expect(within(ping).getByRole('button', { name: 'Sound' })).toBeTruthy();
  });

  it('saves a toggle at once with just that switch and shows the answer', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { ...SETTINGS, toast: { ...SETTINGS.toast, ping: false } } } : { body: SETTINGS }));
    await open();
    const ping = await screen.findByRole('group', { name: 'ping' });
    fireEvent.click(within(ping).getByRole('button', { name: 'Toast' }));
    await waitFor(() => expect(posts(calls, '/api/notify/settings')).toHaveLength(1));
    expect(posts(calls, '/api/notify/settings')[0]?.body).toEqual({ toast: { ping: false } });
    await waitFor(() => expect(within(ping).getByRole('button', { name: 'Toast' }).getAttribute('aria-pressed')).toBe('false'));
  });

  it('says only the page may change the settings when the save is refused', async () => {
    stubApi((call) => (call.method === 'POST' ? { status: 403, body: { error: { code: 'forbidden', message: 'no' } } } : { body: SETTINGS }));
    await open();
    const ping = await screen.findByRole('group', { name: 'ping' });
    fireEvent.click(within(ping).getByRole('button', { name: 'Sound' }));
    expect(await screen.findByText('Only this page may change the settings.')).toBeTruthy();
  });

  it('leaves a sentence instead of a broken row when the settings cannot be read', async () => {
    stubApi(() => ({ status: 500, body: { error: { code: 'internal', message: 'disk' } } }));
    await open();
    expect(await screen.findByText('The settings could not be read: disk.')).toBeTruthy();
  });

  it('says OS notifications are off when notify.os is false', async () => {
    stubApi(() => ({ body: { ...SETTINGS, os: false } }));
    await open();
    expect(await screen.findByText('OS notifications are off in config.json (notify.os): nothing is shown.')).toBeTruthy();
  });
});
