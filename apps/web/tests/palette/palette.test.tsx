/* @layer renderer-app @kind test */
import { act, cleanup, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { resolvePalette } from '../../src/palette';
import { renderAt } from '../helpers/render-at';
import { waitForStream } from '../helpers/wait-for-stream';
import { snapshot } from '../helpers/snapshot';

afterEach(() => {
  cleanup();
  delete document.documentElement.dataset.palette;
});

const board = (name: string, palette?: string) => ({ board: { project: 'x', name, ...(palette ? { palette } : {}) } });

describe('K1 resolvePalette', { tags: ['critical'] }, () => {
  it('maps the built-in project keys and leaves the rest basic', () => {
    expect(resolvePalette('rotp')).toBe('rotp');
    expect(resolvePalette('brock')).toBe('brock');
    expect(resolvePalette('archipelia')).toBe('archipelia');
    expect(resolvePalette('pixelarttool')).toBe('bxel');
    expect(resolvePalette('argus')).toBeUndefined();
    expect(resolvePalette('constructor')).toBeUndefined();
  });

  it('lets the stored setting win, and basic means no attribute', () => {
    expect(resolvePalette('argus', 'brock')).toBe('brock');
    expect(resolvePalette('rotp', 'basic')).toBeUndefined();
    expect(resolvePalette('rotp', 'nonsense')).toBe('rotp');
  });
});

describe('S1 palette attribute', { tags: ['standard'] }, () => {
  it('follows the palette setting of the board card live, and basic clears it', async () => {
    const { sources, portfolio } = renderAt('/p/argus/');
    await waitForStream(sources);
    await waitFor(() => expect(portfolio()).toBeTruthy());
    const card = (palette?: string) => ({ key: 'argus', name: 'Argus', state: 'running', root: null, url: '/p/argus/', ...(palette ? { palette } : {}) });
    act(() => {
      sources[0]?.emit('snapshot', snapshot('argus', 'meta', board('Argus')));
      portfolio()?.emit('snapshot', snapshot('_portfolio', 'cards', { argus: card() }));
    });
    expect(document.documentElement.dataset.palette).toBeUndefined();
    act(() => portfolio()?.emit('changes', { scope: '_portfolio', seq: 11, changes: [{ coll: 'cards', id: 'argus', op: 'set', data: card('brock') }] }));
    await waitFor(() => expect(document.documentElement.dataset.palette).toBe('brock'));
    act(() => portfolio()?.emit('changes', { scope: '_portfolio', seq: 12, changes: [{ coll: 'cards', id: 'argus', op: 'set', data: card('basic') }] }));
    await waitFor(() => expect(document.documentElement.dataset.palette).toBeUndefined());
  });

  it('keeps the overview and its project cards on the basic palette, as the Node page does', async () => {
    const { sources } = renderAt('/');
    await waitForStream(sources);
    const rotp = { key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/' };
    act(() => sources[0]?.emit('snapshot', snapshot('_portfolio', 'cards', { rotp })));
    const open = await screen.findByRole('button', { name: 'Open the RotP board' });
    expect(open.closest('[data-palette]')).toBeNull();
    expect(document.documentElement.dataset.palette).toBeUndefined();
  });
});
