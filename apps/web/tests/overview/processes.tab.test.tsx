/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { CARDS, PROCS, PROCS_META } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';
import type { ApiCall } from './helpers/stub-api';

installHarness();

const open = async (collections: Parameters<typeof feed>[1] = { cards: CARDS, procs: PROCS, meta: PROCS_META }) => {
  const view = renderAt('/#processes');
  await feed(view.sources, collections);
  return view;
};

const kills = (calls: ApiCall[]): ApiCall[] => calls.filter((call) => call.method === 'POST' && call.path.startsWith('/api/procs'));

describe('S1 the process list', { tags: ['standard'] }, () => {
  it('narrows to the processes that need a look', async () => {
    stubApi();
    await open();
    const grid = await screen.findByRole('grid');
    expect(within(grid).getAllByText('explorer').length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole('radio', { name: /Needs a look\s*2/ }));
    await waitFor(() => expect(within(screen.getByRole('grid')).queryByText('explorer')).toBeNull());
    expect(within(screen.getByRole('grid')).getAllByText('vite').length).toBeGreaterThan(0);
  });
});

const ask = async (label = 'Kill vite 4812') => fireEvent.click(await screen.findByRole('button', { name: label }));

describe('K4 killing', { tags: ['standard'] }, () => {
  it('asks under the row first, naming the process and its memory, then posts the pid, the startedAt as sampled and tree false', async () => {
    const calls = stubApi(() => ({ body: { killed: true } }));
    await open();
    await ask();
    expect(await screen.findByText('Kill vite 4812 (1.9 GB)?')).toBeTruthy();
    expect(kills(calls)).toEqual([]);
    fireEvent.click(screen.getByRole('button', { name: 'Kill' }));
    expect(await screen.findByText('Killed vite 4812.')).toBeTruthy();
    expect(kills(calls)).toEqual([{ path: '/api/procs/kill', method: 'POST', body: { pid: 4812, startedAt: PROCS['4812-1'].startedAt, tree: false } }]);
  });

  it('posts tree true for Kill with children and reports the count the server gives back', async () => {
    const calls = stubApi(() => ({ body: { killed: [4812, 4813, 4814] } }));
    await open();
    await ask();
    fireEvent.click(await screen.findByRole('button', { name: 'Kill with children' }));
    expect(await screen.findByText('Killed vite 4812 and its children (3 processes).')).toBeTruthy();
    expect(kills(calls)[0]?.body).toMatchObject({ tree: true });
  });

  it('cancels the question without posting anything', async () => {
    const calls = stubApi();
    await open();
    await ask();
    fireEvent.click(await screen.findByRole('button', { name: 'Cancel' }));
    expect(await screen.findByRole('button', { name: 'Kill vite 4812' })).toBeTruthy();
    expect(screen.queryByText('Kill vite 4812 (1.9 GB)?')).toBeNull();
    expect(kills(calls)).toEqual([]);
  });
});

describe('K4 a protected process and the stale ones of a rule', { tags: ['standard'] }, () => {
  it('shows protected and no Kill for a protected process, and a Kill for the others', async () => {
    stubApi();
    await open();
    const grid = await screen.findByRole('grid');
    expect(within(grid).getAllByLabelText('Protected')).toHaveLength(1);
    expect(screen.queryByRole('button', { name: 'Kill explorer 100' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Kill electron 4900' })).toBeTruthy();
  });

  it('offers Kill stale for a rule with stale killable processes, asks with their count and memory, then posts the rule with idleOnly', async () => {
    const calls = stubApi(() => ({ body: { killed: [4812] } }));
    await open();
    fireEvent.click(await screen.findByRole('button', { name: 'Kill stale (1)' }));
    expect(await screen.findByText('Kill 1 stale node-dev-server process (1.9 GB)?')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Kill' }));
    expect(await screen.findByText('Killed 1 stale node-dev-server process.')).toBeTruthy();
    expect(kills(calls)).toEqual([{ path: '/api/procs/kill-rule', method: 'POST', body: { rule: 'node-dev-server', idleOnly: true } }]);
  });
});
