/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { ago, runningDoc } from './fixtures';
import { mountBuilds } from './mount-builds';
import { board, isOpen, line, logCalls, openTerminal, tailOf, waitForSummary } from './terminal-dom';

cleanAfterEach();

describe('S1 the summary line of a Terminal', { tags: ['standard'] }, () => {
  it('is folded by default, even while running: the run, the newest output and nothing loaded', async () => {
    const { calls } = await mountBuilds(board(), tailOf([]));
    const summary = within(await waitForSummary());
    expect(summary.getByText('R-4 · running · input on')).toBeTruthy();
    expect(summary.getByText('compiling main.ts')).toBeTruthy();
    expect(isOpen()).toBe(false);
    expect(logCalls(calls)).toHaveLength(0);
  });
});

describe('S1 an opened Terminal', { tags: ['standard'] }, () => {
  it('loads the newest 300 lines', async () => {
    const { calls } = await mountBuilds(board(), tailOf([line(1, 'first line'), line(2, 'second line')]));
    await waitForSummary();
    openTerminal();
    expect(await screen.findByText('first line')).toBeTruthy();
    expect(screen.getByText('second line')).toBeTruthy();
    expect(logCalls(calls).map((call) => call.url)).toEqual(['/api/p/rotp/runs/R-4/logs?tail=300']);
  });

  it('starts folded again on a new run of the launcher', async () => {
    const { change } = await mountBuilds(board(), tailOf([line(1, 'first line')]));
    await waitForSummary();
    openTerminal();
    await screen.findByText('first line');
    change('runs', 'R-5', runningDoc('R-5', 'launcher-master', { queuedAt: ago(0), startedAt: ago(0) }));
    expect(await screen.findByText('R-5 · running · input on')).toBeTruthy();
    expect(isOpen()).toBe(false);
  });
});
