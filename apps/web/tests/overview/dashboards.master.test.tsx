/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { wholeText } from '../helpers/whole-text';
import { cards, MASTER, open, posts } from './helpers/dashboards-world';
import { minutesAgo } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';

installHarness();

describe('the master launcher of a card', () => {
  it('says no master set on a card without one', async () => {
    stubApi();
    await open();
    expect(await screen.findByText('no master set')).toBeTruthy();
  });

  it('starts the master launcher in the project scope and says so', async () => {
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { run: { id: 'R-9' }, already: false } } : undefined));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Play' }));
    expect(await screen.findByText('Started RotP master as R-9.')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/p/rotp/runs', method: 'POST', body: { name: 'launcher-master', visible: false } }]);
  });

  it('offers Stop while the master run lives, and stops it in its scope', async () => {
    const live = { id: 'R-9', scope: 'rotp', name: 'launcher-master', status: 'running', queuedAt: minutesAgo(2), startedAt: minutesAgo(1.5), lastLine: 'window open' };
    const calls = stubApi((call) => (call.method === 'POST' ? { body: { run: { id: 'R-9', status: 'stopped' } } } : undefined));
    await open({ cards: { ...cards, rotp: { ...cards.rotp, master: { ...MASTER, lastRun: live } } } });
    expect(await screen.findByText(wholeText(/^\s*running 2m · window open$/))).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Stop' }));
    expect(await screen.findByText('Stopped RotP master (R-9).')).toBeTruthy();
    expect(posts(calls)).toEqual([{ path: '/api/p/rotp/runs/R-9/stop', method: 'POST', body: {} }]);
  });

  it('shows a disabled Play when the launcher has no run definition', async () => {
    stubApi();
    await open({ cards: { ...cards, rotp: { ...cards.rotp, master: { ...MASTER, play: null } } } });
    const play = await screen.findByRole('button', { name: 'Play' });
    expect((play as HTMLButtonElement).disabled).toBe(true);
  });
});
