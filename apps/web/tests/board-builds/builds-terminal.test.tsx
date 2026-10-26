/* @layer renderer-app @kind test */
import { screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { ago, runDoc, runningDoc } from './fixtures';
import { mountBuilds } from './mount-builds';
import { board, isOpen, line, logCalls, openTerminal, summaryOf, tailOf, waitForSummary } from './terminal-dom';

cleanAfterEach();

describe('the summary line of a Terminal', () => {
  it('is folded by default, even while running: the run, the newest output and nothing loaded', async () => {
    const { calls } = await mountBuilds(board(), tailOf([]));
    const summary = within(await waitForSummary());
    expect(summary.getByText('R-4 · running · input on')).toBeTruthy();
    expect(summary.getByText('compiling main.ts')).toBeTruthy();
    expect(isOpen()).toBe(false);
    expect(logCalls(calls)).toHaveLength(0);
  });

  it('says Launching... while a run has said nothing', async () => {
    await mountBuilds(board(runningDoc('R-4', 'launcher-master')));
    expect(within(await waitForSummary()).getByText('Launching...')).toBeTruthy();
  });

  it('reads starting for a queued run', async () => {
    await mountBuilds(board(runDoc('R-4', 'launcher-master', { status: 'queued', startedAt: null, endedAt: null })));
    expect(within(await waitForSummary()).getByText('R-4 · starting')).toBeTruthy();
  });

  it('says how an ended run ended, with its last line', async () => {
    await mountBuilds(board(runDoc('R-4', 'launcher-master', { status: 'failed', exitCode: 3, lastLine: 'boom' })));
    const summary = within(await waitForSummary());
    expect(summary.getByText('R-4 · failed (exit 3)')).toBeTruthy();
    expect(summary.getByText('boom')).toBeTruthy();
  });
});

describe('an opened Terminal', () => {
  it('loads the newest 300 lines', async () => {
    const { calls } = await mountBuilds(board(), tailOf([line(1, 'first line'), line(2, 'second line')]));
    await waitForSummary();
    openTerminal();
    expect(await screen.findByText('first line')).toBeTruthy();
    expect(screen.getByText('second line')).toBeTruthy();
    expect(logCalls(calls).map((call) => call.url)).toEqual(['/api/p/rotp/runs/R-4/logs?tail=300']);
  });

  it('draws colour codes as colours, stderr tinted and system lines dim', async () => {
    await mountBuilds(board(), tailOf([
      line(1, '\u001b[31mred text\u001b[0m and plain'),
      line(2, 'went wrong', 'err'),
      line(3, 'started', 'sys'),
    ]));
    await waitForSummary();
    openTerminal();
    const red = await waitFor(() => {
      const found = document.querySelector('.log-panel__ansi--fg-red');
      expect(found).not.toBeNull();
      return found as Element;
    });
    expect(red.textContent).toBe('red text');
    expect(document.querySelector('.log-panel__msg--err')?.textContent).toBe('went wrong');
    expect(document.querySelector('.log-panel__msg--sys')?.textContent).toBe('started');
  });

  it('closes its stream when folded again', async () => {
    const { streams } = await mountBuilds(board(), tailOf([line(1, 'first line')]));
    await waitForSummary();
    openTerminal();
    await waitFor(() => expect(streams).toHaveLength(1));
    openTerminal();
    await waitFor(() => expect(streams[0]?.closed).toBe(true));
  });

  it('starts folded again on a new run of the launcher', async () => {
    const { change } = await mountBuilds(board(), tailOf([line(1, 'first line')]));
    await waitForSummary();
    openTerminal();
    await screen.findByText('first line');
    change('runs', 'R-5', runningDoc('R-5', 'launcher-master', { queuedAt: ago(0), startedAt: ago(0) }));
    expect(await screen.findByText('R-5 · running · input on')).toBeTruthy();
    expect(isOpen()).toBe(false);
    expect(summaryOf()).toBeTruthy();
  });
});
