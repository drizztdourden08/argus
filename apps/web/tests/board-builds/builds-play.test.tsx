/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { ago, buildDoc, MIN, defDoc, launcherDoc, runDoc, runningDoc } from './fixtures';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

const board = (runs: object[] = [], defExtra: object = {}) => ({
  launchers: [launcherDoc('master')],
  launches: [defDoc('master', defExtra)],
  runs: runs as Record<string, unknown>[],
});

const playButton = () => screen.findByRole<HTMLButtonElement>('button', { name: 'Play master' });

describe('Play of a launcher', () => {
  it('idle: Play is there and enabled, and no run is mentioned', async () => {
    await mountBuilds(board());
    expect((await playButton()).disabled).toBe(false);
    expect(screen.queryByRole('status', { name: /^Run R-/ })).toBeNull();
  });

  it('starts launcher-<name> by name, hidden unless the definition says window', async () => {
    const { calls } = await mountBuilds(board());
    fireEvent.click(await playButton());
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/runs', method: 'POST', body: { name: 'launcher-master', visible: false } });
  });

  it('asks for a console window only when the definition has window', async () => {
    const { calls } = await mountBuilds(board([], { window: true }));
    fireEvent.click(await playButton());
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ name: 'launcher-master', visible: true });
  });

  it('a start the server refuses leaves Play in place', async () => {
    const refuse = (call: { url: string }) => (call.url.endsWith('/runs') ? { status: 409, body: { error: { code: 'conflict', message: 'taken' } } } : undefined);
    const { calls } = await mountBuilds(board(), refuse);
    fireEvent.click(await playButton());
    await waitFor(() => expect(calls).toHaveLength(1));
    await waitFor(() => expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Play master' }).disabled).toBe(false));
  });

  it('with no run definition Play is disabled and says why', async () => {
    await mountBuilds({ launchers: [launcherDoc('master')], launches: [] });
    const play = await playButton();
    expect(play.disabled).toBe(true);
    expect(play.getAttribute('title')).toBe('No Play for launcher master: its command spans several lines, or its run name is taken (coord launcher show master)');
  });
});

describe('Stop and the status line', () => {
  it('running: Stop and the newest output line replace Play', async () => {
    await mountBuilds(board([runningDoc('R-7', 'launcher-master', { lastLine: 'window open' })]));
    expect(await screen.findByRole('button', { name: 'Stop R-7' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Play master' })).toBeNull();
    const line = screen.getByRole('status', { name: 'Run R-7' });
    expect(line.textContent).toContain('running');
    expect(line.textContent).toContain('window open');
  });

  it('running with nothing said yet reads Launching...', async () => {
    await mountBuilds(board([runningDoc('R-7', 'launcher-master')]));
    expect((await screen.findByRole('status', { name: 'Run R-7' })).textContent).toContain('Launching...');
  });

  it('queued says it waits for a slot', async () => {
    await mountBuilds(board([runDoc('R-7', 'launcher-master', { status: 'queued', startedAt: null, endedAt: null })]));
    expect((await screen.findByRole('status', { name: 'Run R-7' })).textContent).toContain('Queued, waiting for a free slot');
  });

  it('Stop stops that run', async () => {
    const { calls } = await mountBuilds(board([runningDoc('R-7', 'launcher-master')]));
    fireEvent.click(await screen.findByRole('button', { name: 'Stop R-7' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/runs/R-7/stop', method: 'POST' });
  });

  it('follows the run live: a change turns Play into Stop and back', async () => {
    const { change } = await mountBuilds(board());
    await playButton();
    change('runs', 'R-7', runningDoc('R-7', 'launcher-master'));
    expect(await screen.findByRole('button', { name: 'Stop R-7' })).toBeTruthy();
    change('runs', 'R-7', runDoc('R-7', 'launcher-master', { status: 'exited', exitCode: 0 }));
    expect(await playButton()).toBeTruthy();
    expect(screen.getByRole('status', { name: 'Run R-7' }).textContent).toContain('exited 0');
  });

  it('finished: how it ended and the last line, with Play again', async () => {
    await mountBuilds(board([runDoc('R-7', 'launcher-master', { status: 'failed', exitCode: 3, lastLine: 'boom' })]));
    const line = await screen.findByRole('status', { name: 'Run R-7' });
    expect(line.textContent).toContain('failed (exit 3)');
    expect(line.textContent).toContain('boom');
    expect(screen.getByRole('button', { name: 'Play master' })).toBeTruthy();
  });

  it('shows only the newest of several finished runs next to Play', async () => {
    const finished = [
      runDoc('R-1', 'launcher-master', { queuedAt: ago(9 * MIN), lastLine: 'oldest' }),
      runDoc('R-2', 'launcher-master', { queuedAt: ago(3 * MIN), lastLine: 'middle' }),
      runDoc('R-3', 'launcher-master', { queuedAt: ago(MIN), lastLine: 'newest' }),
    ];
    await mountBuilds(board(finished));
    const statuses = await screen.findAllByRole('status', { name: /^Run R-/ });
    expect(statuses.map((status) => status.getAttribute('aria-label'))).toEqual(['Run R-3']);
    expect(statuses[0]?.textContent).toContain('newest');
  });

  it('a run of another definition does not show on this launcher', async () => {
    await mountBuilds({ ...board([runningDoc('R-1', 'launcher-other')]), launchers: [launcherDoc('master'), launcherDoc('other')], launches: [defDoc('master'), defDoc('other')] });
    const card = within(await screen.findByRole('group', { name: 'Launcher master: Launcher master' }));
    expect(card.getByRole('button', { name: 'Play master' })).toBeTruthy();
    expect(card.queryByRole('status', { name: /^Run R-/ })).toBeNull();
  });
});

describe('where Play is hidden', () => {
  it('a launcher joined to a draft bundle has no Play and no terminal, but shows its command', async () => {
    await mountBuilds({
      launchers: [launcherDoc('master'), launcherDoc('wt-1', { joined: 'b1', bundle: 'b1' })],
      launches: [defDoc('master'), defDoc('wt-1')],
      runs: [runDoc('R-1', 'launcher-wt-1')],
      builds: [buildDoc('b1', { launcher: 'wt-1', status: 'draft' })],
    });
    expect(await screen.findByText('game run --wt-1')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Play wt-1' })).toBeNull();
    expect(screen.queryByRole('status', { name: 'Run R-1' })).toBeNull();
  });

  it('a retired launcher has no Play at all', async () => {
    await mountBuilds({
      launchers: [launcherDoc('master'), launcherDoc('old-1', { retiredAt: ago(MIN), retiredBy: 'worker' })],
      launches: [defDoc('master')],
    });
    await screen.findByText(/1 to start/);
    expect(screen.queryByRole('button', { name: 'Play old-1' })).toBeNull();
  });
});
