/* @layer renderer-app @kind test */
import { act, cleanup, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { resolvePalette } from '../../src/palette';
import { renderAt } from '../helpers/render-at';
import { snapshot } from '../helpers/snapshot';

afterEach(() => {
  cleanup();
  delete document.documentElement.dataset.palette;
});

const board = (name: string, palette?: string) => ({ board: { project: 'x', name, ...(palette ? { palette } : {}) } });

describe('resolvePalette', () => {
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

describe('palette attribute', () => {
  it('sets data-palette on the document from the key map while a board is open', async () => {
    const { sources } = renderAt('/p/pixelarttool/');
    await waitFor(() => expect(sources).toHaveLength(1));
    act(() => sources[0]?.emit('snapshot', snapshot('pixelarttool', 'meta', board('Bxel'))));
    expect(await screen.findByRole('heading', { name: 'Bxel' })).toBeTruthy();
    expect(document.documentElement.dataset.palette).toBe('bxel');
  });

  it('follows the palette setting of meta/board live, and basic clears it', async () => {
    const { sources } = renderAt('/p/argus/');
    await waitFor(() => expect(sources).toHaveLength(1));
    act(() => sources[0]?.emit('snapshot', snapshot('argus', 'meta', board('Argus'))));
    expect(document.documentElement.dataset.palette).toBeUndefined();
    act(() => sources[0]?.emit('changes', { scope: 'argus', seq: 11, changes: [{ coll: 'meta', id: 'board', op: 'set', data: board('Argus', 'brock').board }] }));
    await waitFor(() => expect(document.documentElement.dataset.palette).toBe('brock'));
    act(() => sources[0]?.emit('changes', { scope: 'argus', seq: 12, changes: [{ coll: 'meta', id: 'board', op: 'set', data: board('Argus', 'basic').board }] }));
    await waitFor(() => expect(document.documentElement.dataset.palette).toBeUndefined());
  });

  it('carries the board palette on the overview card, never on the overview itself', async () => {
    const { sources } = renderAt('/');
    await waitFor(() => expect(sources).toHaveLength(1));
    const rotp = { key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/' };
    act(() => sources[0]?.emit('snapshot', snapshot('_portfolio', 'cards', { rotp })));
    const link = await screen.findByRole('link', { name: 'Open board' });
    expect(link.closest('[data-palette]')?.getAttribute('data-palette')).toBe('rotp');
    expect(document.documentElement.dataset.palette).toBeUndefined();
  });
});
