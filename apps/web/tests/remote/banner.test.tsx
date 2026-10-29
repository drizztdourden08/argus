/* @layer renderer-app @kind test */
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LocalBanner } from '../../src/shell';
import { bannerText } from '../../src/shell/LocalBanner/behavior/banner-text';
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
  expect(screen.queryByRole('status')).toBeNull();
};

describe('the local mode banner text', () => {
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

  it('has no text in remote mode', () => {
    expect(bannerText({ mode: 'remote', replay: { prs: 4, releases: 0, packages: 0, checks: 0 } })).toBeNull();
  });
});

describe('the local mode banner', () => {
  it('shows the replay counts in local mode', async () => {
    stubRemote(local({ prs: 2, releases: 1, packages: 0, checks: 4 }));
    render(<LocalBanner />);
    const line = await screen.findByRole('status');
    expect(line.textContent).toBe('Local mode: 2 pull requests, 1 release, 4 check runs wait for the go');
  });

  it('shows nothing in remote mode', async () => {
    await expectSilent({ '/api/remote': { mode: 'remote', replay: { prs: 2, releases: 0, packages: 0, checks: 0 } } });
  });

  it('shows nothing when the server does not answer', async () => {
    await expectSilent({});
  });

  it('asks again every 60 seconds', async () => {
    const interval = vi.spyOn(window, 'setInterval');
    const calls = stubRemote(local({ prs: 1, releases: 0, packages: 0, checks: 0 }));
    render(<LocalBanner />);
    await screen.findByRole('status');
    expect(interval.mock.calls.some((call) => call[1] === 60_000)).toBe(true);
    expect(calls.every((call) => call.path === '/api/remote')).toBe(true);
  });
});

describe('the pages', () => {
  it('carry the banner on the overview', async () => {
    stubRemote(local({ prs: 0, releases: 2, packages: 0, checks: 0 }));
    renderAt('/');
    expect(await screen.findByText('Local mode: 2 releases wait for the go')).toBeTruthy();
  });
});
