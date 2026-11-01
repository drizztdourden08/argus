/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { open, posts } from './helpers/dashboards-world';
import { minutesAgo } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';

installHarness();

describe('S1 the send down popover', { tags: ['standard'] }, () => {
  it('sends to every lead: the button opens the dialog, the text enables Send down, the post goes to the portfolio route', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-1', status: 'delivered', outcome: null } } : undefined));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog', { name: 'Send down to all projects' });
    expect(within(dialog).getByRole('button', { name: 'Send down' }).hasAttribute('disabled')).toBe(true);
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'ship it' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('O-1 sent down to all projects.')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/portfolio/send', method: 'POST', body: { project: 'all', text: 'ship it', decision: true } }]);
  });

  it('sends a note to one project from its card', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-2', status: 'waiting for a board', outcome: null } } : undefined));
    await open();
    fireEvent.click(await screen.findByTitle('Send a decision or a note to RotP'));
    const dialog = await screen.findByRole('dialog', { name: 'Send down to RotP' });
    fireEvent.click(within(dialog).getByText('Note'));
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'fyi' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('O-2 is kept for RotP until its board exists.')).toBeTruthy();
    expect(posts(calls)[0]?.body).toEqual({ project: 'rotp', text: 'fyi', decision: false });
  });

  it('keeps the popover and says so when the post is refused', async () => {
    stubApi((call) => (call.method === 'POST' ? { body: { id: 'O-3', status: 'refused', outcome: 'too long' } } : undefined));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'x' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Send down' }));
    expect(await screen.findByText('Refused: too long.')).toBeTruthy();
    expect(screen.getByRole('dialog')).toBeTruthy();
  });

  it('refuses a text over 50000 characters with a plain sentence and posts nothing', async () => {
    const calls = stubApi();
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'x'.repeat(50001) } });
    expect(await within(dialog).findByText(/This text is 50001 characters and one post takes 50000 at most/)).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Send down' }).hasAttribute('disabled')).toBe(true);
    expect(posts(calls)).toEqual([]);
  });
});

describe('S1 running now', { tags: ['standard'] }, () => {
  it('lists the live runs of every project', async () => {
    stubApi((call) => (call.path.startsWith('/api/runs')
      ? { body: { runs: [{ id: 'R-5', scope: 'rotp', name: 'hello', status: 'running', queuedAt: minutesAgo(1), startedAt: minutesAgo(1), lastLine: 'compiling' }] } }
      : undefined));
    await open();
    expect(await screen.findByText('hello R-5')).toBeTruthy();
    expect(screen.getByText('compiling')).toBeTruthy();
  });
});
