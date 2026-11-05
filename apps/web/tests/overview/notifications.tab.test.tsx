/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
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

describe('S1 the Notifications tab record', { tags: ['standard'] }, () => {
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
});

describe('S1 the test button', { tags: ['standard'] }, () => {
  it('posts to /api/notify/test and says the toast should be on screen', async () => {
    const calls = stubApi((call) => (call.path === '/api/notify/test' ? { body: { toast: 'shown' } } : { body: SETTINGS }));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send a test notification' }));
    expect(await screen.findByText('Sent: the toast should be on your screen now.')).toBeTruthy();
    expect(posts(calls, '/api/notify/test')).toHaveLength(1);
  });

  it('says not sent with the message when the route fails', async () => {
    stubApi((call) => (call.path === '/api/notify/test' ? { status: 500, body: { error: { code: 'internal', message: 'boom' } } } : { body: SETTINGS }));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send a test notification' }));
    expect(await screen.findByText('Not sent: boom.')).toBeTruthy();
  });
});
