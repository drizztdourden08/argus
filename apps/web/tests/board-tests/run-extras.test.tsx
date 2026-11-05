/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { defDoc, launcherDoc, planDoc, runDoc, runOn, testDoc } from './fixtures';
import { mountTab } from './mount-tab';
import type { Responder } from './mount-tab';

cleanAfterEach();

const NOTE = 'Ship **this** with [notes](https://example.com/n)';

const launched = (runs: object[], run: object = {}, launcher: object = {}) => ({
  group: 'run',
  items: [planDoc('I-9', ['T-1']), runOn('I-50', 'l1', run)],
  launchers: [launcherDoc('l1', launcher)],
  launches: [defDoc('l1')],
  tests: [testDoc('T-1')],
  runs: runs as Record<string, unknown>[],
});

const running = (extra: object = {}) => runDoc('R-4', 'launcher-l1', { status: 'running', endedAt: null, exitCode: null, lastLine: 'compiling main.ts', stdin: true, ...extra });

const tail: Responder = (call) => {
  if (call.url.includes('/attachments')) return { status: 200, body: { attachments: [] } };
  if (!call.url.includes('/logs?')) return undefined;
  return { status: 200, body: { lines: [{ seq: 1, stream: 'out', at: '2026-10-26T10:00:00.000Z', text: 'first line' }], next: 1, dropped: 0 } };
};

describe('S1 the note of a launcher on a run', { tags: ['standard'] }, () => {
  it('is a plain folded line, and rich text once opened', async () => {
    const { container } = await mountTab(launched([], {}, { note: NOTE }));
    const folded = await screen.findByText('Ship this with notes');
    expect(container.querySelector('a[href="https://example.com/n"]')).toBeNull();
    fireEvent.click(folded.closest('summary') as HTMLElement);
    expect(await screen.findByRole('link', { name: /^notes/ })).toBeTruthy();
    expect(container.querySelector('strong')?.textContent).toBe('this');
    fireEvent.click(screen.getByText('Note').closest('summary') as HTMLElement);
    expect(await screen.findByText('Ship this with notes')).toBeTruthy();
  });

  it('is cut to one short line when it is long', async () => {
    await mountTab(launched([], {}, { note: 'word '.repeat(80) }));
    const line = await screen.findByText(/^word word/);
    expect(line.textContent.endsWith('...')).toBe(true);
    expect(line.textContent.length).toBeLessThan(170);
  });
});

describe('S1 the terminal under Play', { tags: ['standard'] }, () => {
  it('is folded by default, even while running: the run and the newest output, nothing loaded', async () => {
    const { calls } = await mountTab(launched([running()]), tail);
    const summary = within((await screen.findByRole('button', { name: 'Terminal' })).parentElement as HTMLElement);
    expect(summary.getByText('R-4 · running · input on')).toBeTruthy();
    expect(summary.getByText('compiling main.ts')).toBeTruthy();
    expect(calls.filter((call) => call.url.includes('/logs'))).toHaveLength(0);
  });

  it('loads the newest lines when the owner opens it', async () => {
    const { calls } = await mountTab(launched([running()]), tail);
    fireEvent.click(await screen.findByRole('button', { name: 'Terminal' }));
    expect(await screen.findByText('first line')).toBeTruthy();
    expect(calls.map((call) => call.url)).toContain('/api/p/rotp/runs/R-4/logs?tail=300');
  });

  it('has no panel without a run, for a run of another definition, or for a draft', async () => {
    const idle = await mountTab(launched([]));
    await screen.findByText('Test number 1');
    expect(screen.queryByText('Terminal')).toBeNull();
    idle.unmount();
    const other = await mountTab(launched([runDoc('R-9', 'launcher-other', { status: 'running', endedAt: null })]));
    await screen.findByText('Test number 1');
    expect(screen.queryByText('Terminal')).toBeNull();
    other.unmount();
    await mountTab(launched([running()], { stage: 'draft' }));
    await screen.findByText('Test number 1');
    expect(screen.queryByText('Terminal')).toBeNull();
  });

  it('opens the full log in a drawer from the Log button', async () => {
    await mountTab(launched([running()]), tail);
    fireEvent.click(await screen.findByRole('button', { name: 'Log of R-4' }));
    const drawer = await screen.findByRole('dialog', { name: 'Run log' });
    await waitFor(() => expect(within(drawer).getByText('first line')).toBeTruthy());
  });
});
