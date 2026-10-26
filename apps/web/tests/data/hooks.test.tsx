/* @layer renderer-app @kind test */
import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { DataProvider, PORTFOLIO_SCOPE, useBoard, useCollection, useDoc, usePortfolio, useStreamStatus } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';

afterEach(cleanup);

const setup = () => {
  const fake = createFakeSources();
  const streams = createScopeStreams(fake.open);
  const wrapper = ({ children }: { children: ReactNode }) => <DataProvider streams={streams}>{children}</DataProvider>;
  return { wrapper, sources: fake.sources };
};

describe('collection hooks', () => {
  it('useCollection returns typed docs with their id once the snapshot is in', async () => {
    const { wrapper, sources } = setup();
    const { result } = renderHook(() => useCollection<{ title: string }>('rotp', 'items'), { wrapper });
    expect(result.current).toEqual({ ready: false, docs: [] });
    await waitFor(() => expect(sources).toHaveLength(1));
    act(() => sources[0]?.emit('snapshot', snapshot('rotp', 'items', { 'I-1': { title: 'one' } })));
    expect(result.current).toEqual({ ready: true, docs: [{ id: 'I-1', title: 'one' }] });
  });

  it('useDoc follows one doc and is null when it is absent', async () => {
    const { wrapper, sources } = setup();
    const { result } = renderHook(() => useDoc<{ name: string }>('rotp', 'meta', 'board'), { wrapper });
    await waitFor(() => expect(sources).toHaveLength(1));
    act(() => sources[0]?.emit('snapshot', snapshot('rotp', 'meta', {})));
    expect(result.current).toEqual({ ready: true, doc: null });
    act(() => sources[0]?.emit('changes', { scope: 'rotp', seq: 11, changes: [{ coll: 'meta', id: 'board', op: 'set', data: { name: 'RotP' } }] }));
    expect(result.current.doc).toEqual({ id: 'board', name: 'RotP' });
  });

  it('useStreamStatus reports the connection', async () => {
    const { wrapper, sources } = setup();
    const { result } = renderHook(() => ({ status: useStreamStatus('rotp'), items: useCollection('rotp', 'items') }), { wrapper });
    expect(result.current.status).toBe('idle');
    await waitFor(() => expect(result.current.status).toBe('connecting'));
    act(() => sources[0]?.emit('ready', { seq: 1 }));
    expect(result.current.status).toBe('live');
  });
});

describe('board and portfolio hooks', () => {
  it('useBoard names the board and hides merged and deleted threads', async () => {
    const { wrapper, sources } = setup();
    const { result } = renderHook(() => useBoard('rotp'), { wrapper });
    expect(result.current.name).toBe('rotp');
    await waitFor(() => expect(sources).toHaveLength(1));
    act(() => {
      sources[0]?.emit('snapshot', snapshot('rotp', 'meta', { board: { name: 'RotP', palette: 'brock' } }));
      sources[0]?.emit('snapshot', snapshot('rotp', 'threads', {
        a: { name: 'a', role: 'thread', status: 'active' },
        b: { name: 'b', role: 'thread', status: 'active', mergedInto: 'a' },
        c: { name: 'c', role: 'thread', status: 'active', deletedAt: 'then' },
      }));
    });
    expect(result.current.ready).toBe(true);
    expect(result.current.name).toBe('RotP');
    expect(result.current.meta?.palette).toBe('brock');
    expect(result.current.threads.map((thread) => thread.id)).toEqual(['a']);
  });

  it('usePortfolio sorts the cards by name', async () => {
    const { wrapper, sources } = setup();
    const { result } = renderHook(() => usePortfolio(), { wrapper });
    await waitFor(() => expect(sources).toHaveLength(1));
    expect(sources[0]?.url).toBe(`/api/p/${PORTFOLIO_SCOPE}/stream?c=cards`);
    act(() => sources[0]?.emit('snapshot', snapshot(PORTFOLIO_SCOPE, 'cards', { b: { key: 'b', name: 'Brock' }, a: { key: 'a', name: 'Argus' } })));
    expect(result.current.cards.map((card) => card.name)).toEqual(['Argus', 'Brock']);
  });
});
