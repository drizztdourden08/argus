/* @layer renderer-app @kind test */
import { act, cleanup, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { snapshot } from '../helpers/snapshot';
import { waitForStream } from '../helpers/wait-for-stream';
import { buildDoc, byId, doneDoc, testDoc } from './fixtures';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const open = async (at: string) => {
  vi.stubGlobal('CSS', { supports: () => false });
  const { sources } = renderAt(at);
  await waitForStream(sources);
  const source = sources[0];
  act(() => {
    source?.emit('snapshot', snapshot('rotp', 'meta', { board: { project: 'rotp', name: 'Relic of the Past' } }));
    source?.emit('snapshot', snapshot('rotp', 'builds', byId([buildDoc('bundle-1')], 'tag')));
    source?.emit('snapshot', snapshot('rotp', 'tests', byId([testDoc('T-1'), testDoc('T-2'), doneDoc('T-3', 'pass', 'ok')])));
  });
};

describe('S1 the Tests tab on the board page', { tags: ['standard'] }, () => {
  it('counts the tests to run in its badge', async () => {
    await open('/p/rotp/#tests');
    const tab = await screen.findByRole('tab', { name: /^Tests/ });
    await waitFor(() => expect(tab.textContent).toContain('2'));
  });

  it('opens the pane of the test an old anchor names, whatever tab the board is on', async () => {
    await open('/p/rotp/#T-2');
    const panes = await screen.findAllByRole('dialog', { name: 'Test detail' });
    expect(panes.length).toBeGreaterThan(0);
  });
});
