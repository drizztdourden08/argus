/* @layer renderer-app @kind test */
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, doneDoc, wakeDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const fresh = () => new Date(Date.now() - 2000).toISOString();

const withWake = (wakes: object[], result: object = {}) => ({
  handed: 'bundle-1',
  builds: [buildDoc('bundle-1')],
  tests: [doneDoc('T-1', 'fail', 'broken', { result: { status: 'fail', text: 'broken', by: 'human', at: fresh(), ...result } })],
  wakes: wakes as Record<string, unknown>[],
});

describe('the wake chip of a done test', () => {
  it('says the thread was told when a listener took the wake', async () => {
    await mountTab(withWake([wakeDoc('W-1', 'T-1', { at: fresh() })]));
    const chip = await screen.findByText('lead notified');
    expect(chip.closest('[title]')?.getAttribute('title')).toContain('lead is listening and was told about this result');
  });

  it('says nobody was listening when the wake reached no one', async () => {
    await mountTab(withWake([wakeDoc('W-1', 'T-1', { status: 'delivered', listeners: 0, at: fresh() })]));
    expect(await screen.findByText('lead not listening')).toBeTruthy();
  });

  it('says waking while a queued wake waits for its window, then not listening', async () => {
    await mountTab(withWake([wakeDoc('W-1', 'T-1', { status: 'queued', listeners: 0, at: fresh() })]));
    expect(await screen.findByText('lead waking...')).toBeTruthy();
  });

  it('speaks through the wake a coalesced one merged into', async () => {
    const merged = [wakeDoc('W-1', 'T-1', { status: 'coalesced', into: 'W-2', at: fresh() }), wakeDoc('W-2', 'T-9', { thread: 'coder', at: fresh() })];
    await mountTab(withWake(merged));
    const chip = await screen.findByText('coder notified');
    expect(chip.closest('[title]')?.getAttribute('title')).toContain('merged into a wake sent a moment earlier');
  });

  it('shows nothing for the wake of an earlier result, for a thread result or without any wake', async () => {
    const early = new Date(Date.now() - 600000).toISOString();
    await mountTab(withWake([wakeDoc('W-1', 'T-1', { at: early })]));
    await screen.findByText('Test T-1');
    expect(screen.queryByText(/notified|not listening|waking/)).toBeNull();
  });

  it('shows nothing when a thread posted the result', async () => {
    await mountTab(withWake([wakeDoc('W-1', 'T-1', { at: fresh() })], { by: 'lead' }));
    await screen.findByText('Test T-1');
    expect(screen.queryByText(/notified|not listening|waking/)).toBeNull();
  });
});
