/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import type { Call, Responder } from '../board-tests/mount-tab';
import { ago, MIN, defDoc, launcherDoc, runDoc, runningDoc } from './fixtures';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

const base = (runs: object[]) => ({
  launchers: [launcherDoc('master')],
  launches: [defDoc('master')],
  runs: runs as Record<string, unknown>[],
});

const rowOf = (container: HTMLElement, id: string): HTMLElement => container.querySelector<HTMLElement>(`[data-run-id="${id}"]`) as HTMLElement;

const shot = (n: number, caption: string) => ({ id: `a${n}`, url: `/files/rotp/a/ra-${n}`, caption, mime: 'image/png', bytes: 1, n, by: 'human', at: ago(MIN) });

const uploads = (calls: Call[]): Call[] => calls.filter((call) => call.method === 'POST');

const answers = (lines: object[] = []): Responder => (call) => {
  if (call.url.includes('/logs?')) return { status: 200, body: { lines, next: lines.length, dropped: 0 } };
  if (call.url.includes('/attachments')) return { status: 200, body: call.method === 'GET' ? { attachments: [shot(1, 'the screen')] } : { attachment: shot(2, 'shot') } };
  return undefined;
};

describe('the run lists', () => {
  it('put live runs under Running now and finished ones under Finished runs, with how each ended', async () => {
    const { container } = await mountBuilds(base([
      runDoc('R-1', 'launcher-master', { queuedAt: ago(30 * MIN) }),
      runDoc('R-2', 'launcher-master', { queuedAt: ago(20 * MIN), status: 'failed', exitCode: 3 }),
      runDoc('R-3', 'launcher-master', { queuedAt: ago(10 * MIN), status: 'stopped', exitCode: null }),
      runDoc('R-4', 'launcher-master', { queuedAt: ago(15 * MIN), status: 'failed', exitCode: null, error: 'failed to start: spawn ENOENT' }),
      runningDoc('R-5', 'launcher-master', { queuedAt: ago(MIN), lastLine: 'serving' }),
    ]));
    expect(await screen.findByText('Running now')).toBeTruthy();
    expect(screen.getByText('Finished runs')).toBeTruthy();
    expect(within(rowOf(container, 'R-1')).getByText('exited 0')).toBeTruthy();
    expect(within(rowOf(container, 'R-2')).getByText('failed (exit 3)')).toBeTruthy();
    expect(within(rowOf(container, 'R-3')).getByText('stopped')).toBeTruthy();
    expect(within(rowOf(container, 'R-4')).getByText('failed (failed to start: spawn ENOENT)')).toBeTruthy();
    expect(within(rowOf(container, 'R-5')).getByText('running')).toBeTruthy();
    expect(within(rowOf(container, 'R-5')).getByText('serving')).toBeTruthy();
  });

  it('show the newest finished first, and keep only the last 20', async () => {
    const runs = Array.from({ length: 25 }, (_, index) => runDoc(`R-${index + 1}`, 'launcher-master', { queuedAt: ago((30 - index) * MIN) }));
    const { container } = await mountBuilds(base(runs));
    await screen.findByText('Finished runs');
    const ids = [...container.querySelectorAll('[data-run-id]')].map((row) => row.getAttribute('data-run-id'));
    expect(ids).toHaveLength(20);
    expect(ids[0]).toBe('R-25');
    expect(ids).not.toContain('R-5');
    expect(ids).toContain('R-6');
  });

  it('list the runs of a launcher whose definition is gone, but not other runs', async () => {
    const { container } = await mountBuilds(base([runDoc('R-1', 'launcher-gone-1'), runDoc('R-2', 'npm-test')]));
    await screen.findByText('Finished runs');
    expect(rowOf(container, 'R-1')).not.toBeNull();
    expect(rowOf(container, 'R-2')).toBeNull();
  });

  it('mark an orphan and say nothing when there is no run', async () => {
    const { container } = await mountBuilds(base([runDoc('R-1', 'launcher-master', { orphan: true, status: 'failed', exitCode: null, error: 'server restarted while running' })]));
    expect(await screen.findByText('orphan')).toBeTruthy();
    expect(rowOf(container, 'R-1')).not.toBeNull();
    const empty = await mountBuilds(base([]));
    expect(empty.container.querySelector('[data-run-id]')).toBeNull();
  });

  it('give a live run a Stop that stops that run', async () => {
    const { calls } = await mountBuilds(base([runningDoc('R-3', 'launcher-master')]));
    fireEvent.click(await screen.findByRole('button', { name: 'Stop run R-3' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/runs/R-3/stop', method: 'POST' });
  });
});

describe('the full log of a run', () => {
  it('opens in a drawer from a row: the run, its output and its screenshots', async () => {
    const { calls } = await mountBuilds(base([runDoc('R-2', 'launcher-master', { status: 'failed', exitCode: 3 })]), answers([{ seq: 1, stream: 'out', at: ago(MIN), text: 'the whole story' }]));
    fireEvent.click(await screen.findByRole('button', { name: 'Open log of R-2' }));
    const drawer = await screen.findByRole('dialog', { name: 'Run log' });
    expect(within(drawer).getByText('launcher-master R-2')).toBeTruthy();
    expect(await within(drawer).findByText('the whole story')).toBeTruthy();
    expect(await within(drawer).findByRole('button', { name: 'Open the screen' })).toBeTruthy();
    expect(calls.map((call) => call.url)).toContain('/api/p/rotp/runs/R-2/logs?tail=4000');
    expect(calls.map((call) => call.url)).toContain('/api/p/rotp/runs/R-2/attachments');
  });

  it('opens from the Log button next to Play, for the run shown there', async () => {
    await mountBuilds(base([runDoc('R-2', 'launcher-master')]), answers());
    fireEvent.click(await screen.findByRole('button', { name: 'Log of R-2' }));
    expect(within(await screen.findByRole('dialog', { name: 'Run log' })).getByText('launcher-master R-2')).toBeTruthy();
  });

  it('has a Stop only for a live run', async () => {
    await mountBuilds(base([runningDoc('R-3', 'launcher-master')]), answers());
    fireEvent.click(await screen.findByRole('button', { name: 'Log of R-3' }));
    const drawer = await screen.findByRole('dialog', { name: 'Run log' });
    expect(within(drawer).getByRole('button', { name: 'Stop run R-3' })).toBeTruthy();
  });

  it('uploads a screenshot to the run with its caption, and shows it', async () => {
    const { calls } = await mountBuilds(base([runDoc('R-2', 'launcher-master')]), answers());
    fireEvent.click(await screen.findByRole('button', { name: 'Log of R-2' }));
    const drawer = await screen.findByRole('dialog', { name: 'Run log' });
    fireEvent.change(within(drawer).getByRole('textbox', { name: 'Caption for R-2' }), { target: { value: 'the crash' } });
    const input = drawer.querySelector('input[type="file"]') as HTMLInputElement;
    fireEvent.change(input, { target: { files: [new File(['x'], 'shot.png', { type: 'image/png' })] } });
    await waitFor(() => expect(uploads(calls)).toHaveLength(1));
    const upload = uploads(calls)[0];
    const url = new URL(upload?.url ?? '', 'http://dashboard');
    expect(url.pathname).toBe('/api/p/rotp/runs/R-2/attachments');
    expect(url.searchParams.get('caption')).toBe('the crash');
    expect(await within(drawer).findByRole('button', { name: 'Open shot' })).toBeTruthy();
  });

  it('refuses an image over 10 MB, without a request', async () => {
    const { calls } = await mountBuilds(base([runDoc('R-2', 'launcher-master')]), answers());
    fireEvent.click(await screen.findByRole('button', { name: 'Log of R-2' }));
    const drawer = await screen.findByRole('dialog', { name: 'Run log' });
    await within(drawer).findByRole('button', { name: 'Open the screen' });
    const input = drawer.querySelector('input[type="file"]') as HTMLInputElement;
    fireEvent.change(input, { target: { files: [new File([new ArrayBuffer(11 * 1024 * 1024)], 'big.png', { type: 'image/png' })] } });
    expect(await within(drawer).findByText('That image is over 10 MB.')).toBeTruthy();
    expect(uploads(calls)).toHaveLength(0);
  });

  it('closes by itself when the run is pruned', async () => {
    const { remove } = await mountBuilds(base([runDoc('R-2', 'launcher-master')]), answers());
    fireEvent.click(await screen.findByRole('button', { name: 'Open log of R-2' }));
    await screen.findByRole('dialog', { name: 'Run log' });
    remove('runs', 'R-2');
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Run log' })).toBeNull());
  });
});
