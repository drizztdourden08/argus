/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { settingsStore } from '../../src/data/notify/settings-store';
import { renderAt } from '../helpers/render-at';
import { feed } from '../overview/helpers/feed';
import { CARDS, NOTIFICATIONS } from '../overview/helpers/fixtures';
import { installHarness } from '../overview/helpers/harness';
import { stubApi } from '../overview/helpers/stub-api';
import type { ApiCall } from '../overview/helpers/stub-api';

const SETTINGS = { os: true, toast: { info: false, ping: true, urgent: true }, sound: { info: false, ping: false, urgent: true } };

installHarness();
beforeEach(() => settingsStore.resetStore());

const posts = (calls: ApiCall[]): ApiCall[] => calls.filter((call) => call.method === 'POST' && call.path === '/api/notify/settings');

const openNotifications = async () => {
  const view = renderAt('/');
  await feed(view.sources, { cards: CARDS, notifications: NOTIFICATIONS });
  fireEvent.click(await screen.findByRole('button', { name: 'Settings' }));
  fireEvent.click(await screen.findByRole('radio', { name: 'Notifications' }));
  return view;
};

describe('S1 the notification settings in the Settings dialog', { tags: ['standard'] }, () => {
  it('saves a toggle at once with just that switch and shows the answer', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { ...SETTINGS, toast: { ...SETTINGS.toast, ping: false } } } : { body: SETTINGS }));
    await openNotifications();
    fireEvent.click(await screen.findByRole('switch', { name: 'Ping toast' }));
    await waitFor(() => expect(posts(calls)).toHaveLength(1));
    expect(posts(calls)[0]?.body).toEqual({ toast: { ping: false } });
    await waitFor(() => expect((screen.getByRole('switch', { name: 'Ping toast' }) as HTMLInputElement).checked).toBe(false));
  });

  it('shows the refusal on the level row when the save is refused', async () => {
    stubApi((call) => (call.method === 'POST' ? { status: 403, body: { error: { code: 'forbidden', message: 'no' } } } : { body: SETTINGS }));
    await openNotifications();
    fireEvent.click(await screen.findByRole('switch', { name: 'Ping sound' }));
    expect(await screen.findByText('Only this page may change the settings.')).toBeTruthy();
  });

  it('leaves a sentence instead of a broken row when the settings cannot be read', async () => {
    stubApi(() => ({ status: 500, body: { error: { code: 'internal', message: 'disk' } } }));
    await openNotifications();
    expect(await screen.findByText('The settings could not be read: disk.')).toBeTruthy();
  });
});
