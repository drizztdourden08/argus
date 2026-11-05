/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LocalBanner } from '../../src/shell';
import { bannerText } from '../../src/shell/LocalBanner/behavior/banner-text';
import { pillText } from '../../src/shell/LocalBanner/behavior/pill-text';
import { renderAt } from '../helpers/render-at';
import { stubRemote } from './helpers/stub-remote';

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const local = (replay: object) => ({ '/api/remote': { mode: 'local', replay } });

const expectSilent = async (routes: Record<string, unknown>) => {
  const calls = stubRemote(routes);
  render(<LocalBanner />);
  await waitFor(() => expect(calls).toHaveLength(1));
  await Promise.resolve();
  expect(screen.queryByRole('button', { name: /^Local/ })).toBeNull();
};

const openPill = async (name: string): Promise<HTMLElement> => {
  fireEvent.click(await screen.findByRole('button', { name }));
  return screen.findByRole('dialog', { name: 'Local mode' });
};

describe('K1 the local mode banner text', { tags: ['critical'] }, () => {
  it('names the parts that wait and leaves out the zero ones', () => {
    expect(bannerText({ mode: 'local', replay: { prs: 2, releases: 0, packages: 3, checks: 1 } }))
      .toBe('Local mode: 2 pull requests, 3 packages, 1 check run wait for the go');
  });

  it('uses the singular for one record', () => {
    expect(bannerText({ mode: 'local', replay: { prs: 1, releases: 0, packages: 0, checks: 0 } })).toBe('Local mode: 1 pull request waits for the go');
    expect(bannerText({ mode: 'local', replay: { prs: 0, releases: 1, packages: 0, checks: 0 } })).toBe('Local mode: 1 release waits for the go');
  });

  it('says nothing waits when every count is zero', () => {
    expect(bannerText({ mode: 'local', replay: { prs: 0, releases: 0, packages: 0, checks: 0 } })).toBe('Local mode: nothing waits for the go');
  });

  it('adds the integration checkout of the board in local mode', () => {
    const info = { mode: 'local' as const, replay: {}, ciRoots: { brock: 'X:/brock' } };
    expect(bannerText(info, 'brock')).toBe('Local mode: nothing waits for the go. CI from X:/brock');
    expect(bannerText(info, 'argus')).toBe('Local mode: nothing waits for the go');
    expect(bannerText(info)).toBe('Local mode: nothing waits for the go');
  });

  it('has no text in remote mode', () => {
    expect(bannerText({ mode: 'remote', replay: { prs: 4, releases: 0, packages: 0, checks: 0 } })).toBeNull();
  });

  it('puts the pull requests and the releases on the pill, with the singular for one', () => {
    expect(pillText({ mode: 'local', replay: { prs: 74, releases: 4, packages: 3 } })).toBe('Local · 74 PRs · 4 releases');
    expect(pillText({ mode: 'local', replay: { prs: 1, releases: 1 } })).toBe('Local · 1 PR · 1 release');
    expect(pillText({ mode: 'local', replay: {} })).toBe('Local · 0 PRs · 0 releases');
  });
});

describe('S1 the local mode pill', { tags: ['standard'] }, () => {
  it('shows the pull requests and releases on the pill and the full text in its popover', async () => {
    stubRemote(local({ prs: 2, releases: 1, packages: 0, checks: 4 }));
    render(<LocalBanner />);
    const panel = await openPill('Local · 2 PRs · 1 release');
    expect(within(panel).getByRole('status').textContent).toBe('Local mode: 2 pull requests, 1 release, 4 check runs wait for the go');
  });

  it('shows nothing in remote mode', async () => {
    await expectSilent({ '/api/remote': { mode: 'remote', replay: { prs: 2, releases: 0, packages: 0, checks: 0 } } });
  });

  it('shows nothing when the server does not answer', async () => {
    await expectSilent({});
  });
});

describe('S1 the local mode pill on a board', { tags: ['standard'] }, () => {
  const answers = {
    '/api/remote': { mode: 'local', replay: {}, ciRoots: { brock: 'X:/brock' }, syncs: { brock: { at: new Date().toISOString(), by: 'lead', note: 'pushed' } } },
  };

  const renderBoard = (key: string) => render(
    <MemoryRouter initialEntries={[`/p/${key}`]}>
      <Routes><Route path="/p/:key" element={<LocalBanner />} /></Routes>
    </MemoryRouter>,
  );

  it('names the integration checkout of the board it shows', async () => {
    stubRemote(answers);
    renderBoard('brock');
    const panel = await openPill('Local · 0 PRs · 0 releases');
    expect(within(panel).getByRole('status').textContent).toBe('Local mode: nothing waits for the go. CI from X:/brock');
  });

  it('holds the last remote sync of the board in the popover', async () => {
    stubRemote(answers);
    renderBoard('brock');
    const panel = await openPill('Local · 0 PRs · 0 releases');
    expect(within(panel).getByText(/last remote sync/)).toBeTruthy();
  });
});

describe('S2 the pages', { tags: ['standard'] }, () => {
  it('carry the pill on the overview', async () => {
    stubRemote(local({ prs: 0, releases: 2, packages: 0, checks: 0 }));
    renderAt('/');
    const panel = await openPill('Local · 0 PRs · 2 releases');
    expect(within(panel).getByText('Local mode: 2 releases wait for the go')).toBeTruthy();
  });
});
