/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { buildDoc, defDoc, launcherDoc, runDoc, runningDoc } from './fixtures';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

const board = (runs: object[] = [], defExtra: object = {}) => ({
  launchers: [launcherDoc('master')],
  launches: [defDoc('master', defExtra)],
  runs: runs as Record<string, unknown>[],
});

const playButton = () => screen.findByRole<HTMLButtonElement>('button', { name: 'Play master' });

describe('S1 Play of a launcher', { tags: ['standard'] }, () => {
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

  it('with no run definition Play is disabled and says why', async () => {
    await mountBuilds({ launchers: [launcherDoc('master')], launches: [] });
    const play = await playButton();
    expect(play.disabled).toBe(true);
    expect(play.getAttribute('title')).toBe('No Play for launcher master: its command spans several lines, or its run name is taken (coord launcher show master)');
  });
});

describe('S1 Stop and the status line', { tags: ['standard'] }, () => {
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
});

describe('I-215 Running now at the top of Builds', { tags: ['standard'] }, () => {
  it('lists the live runs and leaves the finished ones to the build pane', async () => {
    await mountBuilds(board([runningDoc('R-7', 'launcher-master'), runDoc('R-6', 'launcher-master')]));
    const section = await screen.findByRole('region', { name: 'Running now' });
    expect(within(section).getByText('R-7')).toBeTruthy();
    expect(within(section).queryByText('R-6')).toBeNull();
  });

  it('shows no section without a live run', async () => {
    await mountBuilds(board([runDoc('R-6', 'launcher-master')]));
    await playButton();
    expect(screen.queryByRole('region', { name: 'Running now' })).toBeNull();
  });
});

describe('K4 where Play is hidden', { tags: ['standard'] }, () => {
  it('a launcher joined to a draft bundle has no Play', async () => {
    await mountBuilds({
      launchers: [launcherDoc('master'), launcherDoc('wt-1', { joined: 'b1', bundle: 'b1' })],
      launches: [defDoc('master'), defDoc('wt-1')],
      runs: [runDoc('R-1', 'launcher-wt-1')],
      builds: [buildDoc('b1', { launcher: 'wt-1', status: 'draft' })],
    });
    await screen.findByText('Master build');
    expect(screen.queryByRole('button', { name: 'Play wt-1' })).toBeNull();
  });
});
