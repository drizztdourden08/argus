/* @layer renderer-app @kind test */
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { ago, MIN, runDoc, runningDoc } from './fixtures';
import { mountBuilds } from './mount-builds';
import { board, line, openTerminal, stdinCalls, typeIntoTerminal, tailOf, waitForSummary } from './terminal-dom';

cleanAfterEach();

describe('following a live run', () => {
  it('follows the stream from where the loaded lines end and shows each line once', async () => {
    const { streams } = await mountBuilds(board(), tailOf([line(1, 'one'), line(2, 'two'), line(3, 'three')]));
    await waitForSummary();
    openTerminal();
    await screen.findByText('three');
    await waitFor(() => expect(streams).toHaveLength(1));
    expect(streams[0]?.url).toBe('/api/p/rotp/runs/R-4/logs/stream?after=3');
    act(() => streams[0]?.emit('log', line(4, 'live four')));
    expect(await screen.findByText('live four')).toBeTruthy();
    act(() => {
      streams[0]?.emit('log', line(4, 'live four'));
      streams[0]?.emit('log', line(5, 'live five'));
    });
    expect(await screen.findByText('live five')).toBeTruthy();
    expect(screen.getAllByText('live four')).toHaveLength(1);
  });

  it('closes the stream after the final status and keeps it open for a plain update', async () => {
    const { streams } = await mountBuilds(board(), tailOf([line(1, 'one')]));
    await waitForSummary();
    openTerminal();
    await waitFor(() => expect(streams).toHaveLength(1));
    act(() => streams[0]?.emit('status', { run: { id: 'R-4', status: 'running' } }));
    expect(streams[0]?.closed).toBe(false);
    act(() => streams[0]?.emit('status', { run: { id: 'R-4', status: 'exited', exitCode: 0 } }));
    expect(streams[0]?.closed).toBe(true);
  });

  it('does not follow a run that ended', async () => {
    const { streams } = await mountBuilds(board(runDoc('R-4', 'launcher-master')), tailOf([line(1, 'done line')]));
    await waitForSummary();
    openTerminal();
    expect(await screen.findByText('done line')).toBeTruthy();
    expect(streams).toHaveLength(0);
  });

  it('says a failed load in the terminal', async () => {
    await mountBuilds(board(), () => ({ status: 500, body: { error: { code: 'unavailable', message: 'the dashboard is gone' } } }));
    await waitForSummary();
    openTerminal();
    expect(await screen.findByText('could not load the output: the dashboard is gone')).toBeTruthy();
  });

  it('says how many older lines were dropped', async () => {
    await mountBuilds(board(runDoc('R-4', 'launcher-master')), tailOf([line(5, 'tail')], 5, 12));
    await waitForSummary();
    openTerminal();
    expect(await screen.findByText('... 12 older lines were dropped to keep the log small')).toBeTruthy();
  });

  it('Clear view empties what is drawn and says the log keeps everything', async () => {
    await mountBuilds(board(), tailOf([line(1, 'first line')]));
    await waitForSummary();
    openTerminal();
    await screen.findByText('first line');
    fireEvent.click(screen.getByRole('button', { name: 'Clear view' }));
    expect(await screen.findByText('view cleared; the log keeps everything')).toBeTruthy();
    expect(screen.queryByText('first line')).toBeNull();
  });
});

describe('the input row of a terminal', () => {
  const inputLabel = 'Send a line to the process';

  it('sends a line to the process on Enter and clears the box', async () => {
    const { calls } = await mountBuilds(board(), tailOf([]));
    const input = await typeIntoTerminal('help me');
    await waitFor(() => expect(stdinCalls(calls)).toHaveLength(1));
    expect(stdinCalls(calls)[0]).toMatchObject({ url: '/api/p/rotp/runs/R-4/stdin', method: 'POST', body: { text: 'help me' } });
    expect(input.value).toBe('');
  });

  it.each([
    ['has no stdin', runningDoc('R-4', 'launcher-master', { stdin: false }), 'This run has no input'],
    ['is over', runDoc('R-4', 'launcher-master'), 'The run is over'],
  ])('is off, with its reason, when the run %s', async (_name, run, reason) => {
    await mountBuilds(board(run), tailOf([line(1, 'one')]));
    await waitForSummary();
    openTerminal();
    await screen.findByText('one');
    const input = screen.getByLabelText<HTMLInputElement>(inputLabel);
    expect(input.disabled).toBe(true);
    expect(input.placeholder).toBe(reason);
  });

  it('keeps the typed text when the process refuses the line, and says why', async () => {
    const tail = tailOf([]);
    const { calls } = await mountBuilds(board(), (call) => (call.url.endsWith('/stdin') ? { status: 409, body: { error: 'run has no stdin' } } : tail(call)));
    const input = await typeIntoTerminal('help me');
    await waitFor(() => expect(stdinCalls(calls)).toHaveLength(1));
    expect(await screen.findByText(/Could not send to R-4/)).toBeTruthy();
    expect(input.value).toBe('help me');
  });

  it('follows the run: off when it ends', async () => {
    const { change } = await mountBuilds(board(), tailOf([line(1, 'one')]));
    await waitForSummary();
    openTerminal();
    expect(await screen.findByLabelText(inputLabel)).toBeTruthy();
    change('runs', 'R-4', runDoc('R-4', 'launcher-master', { queuedAt: ago(5 * MIN) }));
    await waitFor(() => expect(screen.getByLabelText<HTMLInputElement>(inputLabel).disabled).toBe(true));
  });
});
