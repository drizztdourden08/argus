/* @layer renderer-app @kind test */
import { cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
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

describe('the Processes tab before and without data', () => {
  it('says Argus has not sampled yet when nothing arrived, and that nothing matches when the sample is empty', async () => {
    stubApi();
    renderAt('/#processes');
    expect(screen.getByText('Argus has not sampled the processes yet.')).toBeTruthy();
    cleanup();
    await open({ procs: {}, meta: { procs: { count: 0, sampledAt: new Date().toISOString() } } });
    expect(await screen.findByText('No process matches a rule right now.')).toBeTruthy();
  });

  it('shows the sampling error in a callout, clipped', async () => {
    stubApi();
    await open({ procs: {}, meta: { procs: { count: 0, error: 'powershell exited with code 1' } } });
    expect(await screen.findByText('Sampling error: powershell exited with code 1')).toBeTruthy();
  });
});

describe('the summary, the rule strip and the rules', () => {
  it('shows the count, the memory, how many need a look and the platform', async () => {
    stubApi();
    await open();
    expect(await screen.findByText('need a look')).toBeTruthy();
    expect(screen.getAllByText('4.2 GB').length).toBeGreaterThan(0);
    expect(screen.getByText('win32')).toBeTruthy();
  });

  it('shows each rule with its count and memory, and says over the cap when the group is past it', async () => {
    stubApi();
    await open();
    expect(await screen.findByRole('button', { name: /node-dev-server\s*1\s*1\.9 GB \/ cap 1\.5 GB/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: /electron-dev\s*1\s*1\.6 GB/ })).toBeTruthy();
  });

  it('lists the rules read-only in a section that carries their number', async () => {
    stubApi();
    await open();
    expect(await screen.findByText('Rules (3)')).toBeTruthy();
  });

  it('lists a row per process with its name and rule', async () => {
    stubApi();
    await open();
    const grid = await screen.findByRole('grid');
    expect(within(grid).getAllByText('vite').length).toBeGreaterThan(0);
    expect(within(grid).getAllByText('electron').length).toBeGreaterThan(0);
    expect(within(grid).getAllByText('explorer').length).toBeGreaterThan(0);
  });

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

describe('killing', () => {
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

  it('says what a 403 and a 404 mean in a toast', async () => {
    let status = 403;
    stubApi(() => ({ status, body: { error: { code: 'x', message: 'no' } } }));
    await open();
    await ask();
    fireEvent.click(await screen.findByRole('button', { name: 'Kill' }));
    expect(await screen.findByText('Argus will not kill vite 4812: it is protected, or its rule does not allow kills.')).toBeTruthy();
    status = 404;
    await ask();
    fireEvent.click(await screen.findByRole('button', { name: 'Kill' }));
    expect(await screen.findByText('vite 4812 is already gone, nothing was killed.')).toBeTruthy();
  });
});

describe('killing a protected process and the stale ones of a rule', () => {
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

  it('offers no Kill stale for a rule with nothing stale', async () => {
    stubApi();
    await open();
    await screen.findByRole('button', { name: /electron-dev\s*1\s*1\.6 GB/ });
    expect(screen.queryByRole('button', { name: 'Kill stale (0)' })).toBeNull();
    expect(screen.getAllByRole('button', { name: /Kill stale/ })).toHaveLength(1);
  });
});

describe('the cells of the process table', () => {
  const sample = (status: string) => ({ ...PROCS['4812-1'], status, flags: [] });

  it('writes memory as Node does (1.9 GB), CPU with its unit, the age and idle time in hours and minutes, and the pid beside the name', async () => {
    stubApi();
    await open();
    const grid = await screen.findByRole('grid');
    expect(within(grid).getAllByText('1.9 GB').length).toBeGreaterThan(0);
    expect(within(grid).getAllByText('12%').length).toBeGreaterThan(0);
    expect(within(grid).getAllByText('3h 10m').length).toBeGreaterThan(0);
    expect(within(grid).getAllByText('4812').length).toBeGreaterThan(0);
  });

  it('marks the rows that need a look with the warning bar and leaves the others plain', async () => {
    stubApi();
    await open({ cards: CARDS, procs: { a: sample('hog'), b: { ...sample('ok'), pid: 2 } }, meta: PROCS_META });
    const grid = await screen.findByRole('grid');
    await waitFor(() => expect(grid.querySelectorAll('[data-state="warning"]').length).toBe(1));
  });

  it('tones each status like the Node page: warning for hog, stale and old, danger for orphan and hung, calm for an unknown one', async () => {
    stubApi();
    await open({ cards: CARDS, procs: { a: sample('hog'), b: { ...sample('old'), pid: 2 }, c: { ...sample('orphan'), pid: 3 }, d: { ...sample('hung'), pid: 4 }, e: { ...sample('exotic'), pid: 5 } }, meta: PROCS_META });
    const grid = await screen.findByRole('grid');
    const tag = (title: string): string => within(grid).getByTitle(title).className;
    expect(tag('Uses more memory than its rule allows.')).toContain('status--warning');
    expect(tag('Running for longer than its rule allows.')).toContain('status--warning');
    expect(tag('Its parent process is gone.')).toContain('status--danger');
    expect(tag('Not responding.')).toContain('status--danger');
    expect(tag('exotic')).toContain('status--neutral');
  });
});
