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

const shot = (n: number, caption: string) => ({ id: `a${n}`, url: `/files/rotp/a/ra-${n}`, caption, mime: 'image/png', bytes: 1, n, by: 'human', at: ago(MIN) });

const uploads = (calls: Call[]): Call[] => calls.filter((call) => call.method === 'POST');

const answers = (lines: object[] = []): Responder => (call) => {
  if (call.url.includes('/logs?')) return { status: 200, body: { lines, next: lines.length, dropped: 0 } };
  if (call.url.includes('/attachments')) return { status: 200, body: call.method === 'GET' ? { attachments: [shot(1, 'the screen')] } : { attachment: shot(2, 'shot') } };
  return undefined;
};

describe('the full log of a run', () => {
  it('opens in a drawer from the Full log of the Terminal: the run, its output and its screenshots', async () => {
    const { calls } = await mountBuilds(base([runDoc('R-2', 'launcher-master', { status: 'failed', exitCode: 3 })]), answers([{ seq: 1, stream: 'out', at: ago(MIN), text: 'the whole story' }]));
    fireEvent.click(await screen.findByRole('button', { name: 'Full log of R-2' }));
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
    fireEvent.click(await screen.findByRole('button', { name: 'Full log of R-2' }));
    await screen.findByRole('dialog', { name: 'Run log' });
    remove('runs', 'R-2');
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Run log' })).toBeNull());
  });
});
