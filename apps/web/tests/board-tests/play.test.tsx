/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, defDoc, launcherDoc, runDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const playable = (runs: object[] = [], defExtra: object = {}) => ({
  builds: [buildDoc('b1', { launcher: 'l1' })],
  launchers: [launcherDoc('l1')],
  launches: [defDoc('l1', defExtra)],
  tests: [testDoc('T-1', { build: 'b1' })],
  runs: runs as Record<string, unknown>[],
});

const running = (extra: object = {}) => runDoc('R-1', 'launcher-l1', { status: 'running', endedAt: null, ...extra });

describe('starting and stopping the Play of a bundle', () => {
  it('idle: Play is there, enabled, and nothing about a run', async () => {
    await mountTab(playable());
    const play = await screen.findByRole<HTMLButtonElement>('button', { name: 'Play b1' });
    expect(play.disabled).toBe(false);
    expect(screen.queryByRole('status', { name: /^Run R-/ })).toBeNull();
  });

  it('clicking Play starts the definition by name, hidden unless the definition says window', async () => {
    const { calls } = await mountTab(playable());
    fireEvent.click(await screen.findByRole('button', { name: 'Play b1' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/runs', method: 'POST', body: { name: 'launcher-l1', visible: false } });
  });

  it('asks for a console window only when the definition has window', async () => {
    const { calls } = await mountTab(playable([], { window: true }));
    fireEvent.click(await screen.findByRole('button', { name: 'Play b1' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ name: 'launcher-l1', visible: true });
  });

  it('clicking Stop stops that run', async () => {
    const { calls } = await mountTab(playable([running()]));
    fireEvent.click(await screen.findByRole('button', { name: 'Stop R-1' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/runs/R-1/stop', method: 'POST' });
  });

  it('follows the run live: a change turns Play into Stop', async () => {
    const { change } = await mountTab(playable());
    expect(await screen.findByRole('button', { name: 'Play b1' })).toBeTruthy();
    change('runs', 'R-7', runDoc('R-7', 'launcher-l1', { status: 'running', endedAt: null }));
    expect(await screen.findByRole('button', { name: 'Stop R-7' })).toBeTruthy();
  });

  it('a start refused by the server leaves Play in place', async () => {
    const refuse = (call: { url: string }) => (call.url.endsWith('/runs') ? { status: 409, body: { error: { code: 'conflict', message: 'taken' } } } : undefined);
    const { calls } = await mountTab(playable(), refuse);
    fireEvent.click(await screen.findByRole('button', { name: 'Play b1' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    await waitFor(() => expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Play b1' }).disabled).toBe(false));
  });
});

describe('the status line next to Play', () => {
  it('running: Stop and the newest output line replace Play', async () => {
    await mountTab(playable([running({ lastLine: 'listening on 3000' })]));
    expect(await screen.findByRole('button', { name: 'Stop R-1' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Play b1' })).toBeNull();
    const line = screen.getByRole('status', { name: 'Run R-1' });
    expect(line.textContent).toContain('running');
    expect(line.textContent).toContain('listening on 3000');
  });

  it('running with nothing said yet reads Launching...', async () => {
    await mountTab(playable([running()]));
    expect((await screen.findByRole('status', { name: 'Run R-1' })).textContent).toContain('Launching...');
  });

  it('queued says it waits for a slot', async () => {
    await mountTab(playable([runDoc('R-1', 'launcher-l1', { status: 'queued', startedAt: null, endedAt: null })]));
    expect((await screen.findByRole('status', { name: 'Run R-1' })).textContent).toContain('Queued, waiting for a free slot');
  });

  it('a run of another definition does not show on this card', async () => {
    await mountTab(playable([runDoc('R-1', 'launcher-other', { status: 'running', endedAt: null })]));
    expect(await screen.findByRole('button', { name: 'Play b1' })).toBeTruthy();
    expect(screen.queryByRole('status', { name: /^Run R-/ })).toBeNull();
  });

  it('finished: how it ended and the last line, with Play again', async () => {
    await mountTab(playable([runDoc('R-1', 'launcher-l1', { status: 'failed', exitCode: 3, lastLine: 'boom' })]));
    const line = await screen.findByRole('status', { name: 'Run R-1' });
    expect(line.textContent).toContain('failed (exit 3)');
    expect(line.textContent).toContain('boom');
    expect(screen.getByRole('button', { name: 'Play b1' })).toBeTruthy();
  });

  it('shows the newest of several finished runs', async () => {
    await mountTab(playable([
      runDoc('R-1', 'launcher-l1', { queuedAt: '2026-10-25T00:00:00.000Z', lastLine: 'older' }),
      runDoc('R-2', 'launcher-l1', { queuedAt: '2026-10-26T00:00:00.000Z', lastLine: 'newer' }),
    ]));
    expect((await screen.findByRole('status', { name: 'Run R-2' })).textContent).toContain('newer');
    expect(screen.queryByRole('status', { name: 'Run R-1' })).toBeNull();
  });
});
