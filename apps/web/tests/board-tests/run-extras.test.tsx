/* @layer renderer-app @kind test */
import { screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { defDoc, launcherDoc, planDoc, runDoc, runOn, testDoc } from './fixtures';
import { choose, openMenu } from './menu-dom';
import { mountTab } from './mount-tab';
import type { Responder } from './mount-tab';

cleanAfterEach();

const launched = (runs: object[], run: object = {}) => ({
  group: 'run',
  items: [planDoc('I-9', ['T-1']), runOn('I-50', 'l1', run)],
  launchers: [launcherDoc('l1')],
  launches: [defDoc('l1')],
  tests: [testDoc('T-1')],
  runs: runs as Record<string, unknown>[],
});

const running = () => runDoc('R-4', 'launcher-l1', { status: 'running', endedAt: null, exitCode: null, lastLine: 'compiling main.ts', stdin: true });

const tail: Responder = (call) => {
  if (call.url.includes('/attachments')) return { status: 200, body: { attachments: [] } };
  if (!call.url.includes('/logs?')) return undefined;
  return { status: 200, body: { lines: [{ seq: 1, stream: 'out', at: '2026-10-26T10:00:00.000Z', text: 'first line' }], next: 1, dropped: 0 } };
};

describe('S1 the run menu', { tags: ['standard'] }, () => {
  it('opens the full log of the newest run in a drawer', async () => {
    await mountTab(launched([running()]), tail);
    await openMenu('TR-50');
    await choose('Log of R-4');
    const drawer = await screen.findByRole('dialog', { name: 'Run log' });
    await waitFor(() => expect(within(drawer).getByText('first line')).toBeTruthy());
  });

  it('has no log item for a draft', async () => {
    await mountTab(launched([running()], { stage: 'draft' }));
    await openMenu('TR-50');
    expect(await screen.findByRole('menuitem', { name: 'Open build' })).toBeTruthy();
    expect(screen.queryByRole('menuitem', { name: 'Log of R-4' })).toBeNull();
  });
});
