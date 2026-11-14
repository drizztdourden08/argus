/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from '../overview/helpers/feed';
import { CARDS, NOTIFICATIONS } from '../overview/helpers/fixtures';
import { installHarness } from '../overview/helpers/harness';
import { stubApi } from '../overview/helpers/stub-api';

const KEY = 'sk-ant-api03-secret-abcd';
const NONE = { set: false, last4: null, savedAt: null };
const SAVED = { set: true, last4: 'abcd', savedAt: '2026-11-14T10:00:00.000Z' };

installHarness();

const openSystem = async () => {
  const view = renderAt('/');
  await feed(view.sources, { cards: CARDS, notifications: NOTIFICATIONS });
  fireEvent.click(await screen.findByRole('button', { name: 'Settings' }));
  fireEvent.click(await screen.findByRole('radio', { name: 'System' }));
};

describe('S1 the Overlord API key in the Settings dialog', { tags: ['standard'] }, () => {
  it('shows only the last 4 characters after a save and clears the field', async () => {
    let saved = false;
    const calls = stubApi((call) => {
      if (call.path !== '/api/overlord/key') return undefined;
      if (call.method === 'PUT') saved = true;
      return { body: saved ? SAVED : NONE };
    });
    await openSystem();
    expect(await screen.findByText('No key saved')).toBeTruthy();
    const field = screen.getByLabelText<HTMLInputElement>('Anthropic API key');
    fireEvent.change(field, { target: { value: KEY } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('****abcd')).toBeTruthy();
    await waitFor(() => expect(field.value).toBe(''));
    expect(calls.some((call) => call.method === 'PUT' && JSON.stringify(call.body) === JSON.stringify({ key: KEY }))).toBe(true);
    expect(screen.queryByText(KEY)).toBeNull();
  });
});
