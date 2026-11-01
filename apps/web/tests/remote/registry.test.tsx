/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { renderAt } from '../helpers/render-at';
import { ago, MIN } from './helpers/fixtures';
import { stubRemote } from './helpers/stub-remote';

cleanAfterEach();

const REPORT = {
  builds: [
    { name: '@drizztdourden08/brock-core', version: '0.115.5', commit: 'abcdef1234567', checkout: 'X:/brock', packedAt: ago(3 * MIN), tarball: 'core.tgz', size: 5 * 1024 * 1024 },
    { name: '@drizztdourden08/tessera', version: '0.92.1', commit: '7654321fedcba', checkout: 'X:/tessera', packedAt: ago(90 * MIN), tarball: 'tessera.tgz', size: 2048 },
  ],
  consumers: [
    { path: 'X:/rotp', packages: ['@drizztdourden08/brock-core', '@drizztdourden08/tessera'], on: true },
    { path: 'X:/bxel', packages: ['@drizztdourden08/brock-core'], on: false },
  ],
  scopeRedirected: false,
};

const open = (report: object): void => {
  stubRemote({ '/api/registry': report });
  renderAt('/#registry');
};

describe('S1 the overview Registry tab', { tags: ['standard'] }, () => {
  it('lists the local builds with package, version, short commit, checkout, age and size', async () => {
    open(REPORT);
    expect((await screen.findAllByText('@drizztdourden08/brock-core')).length).toBeGreaterThan(0);
    expect(screen.getAllByText('0.115.5').length).toBeGreaterThan(0);
    expect(screen.getAllByText('abcdef1').length).toBeGreaterThan(0);
    expect(screen.getAllByText('X:/brock').length).toBeGreaterThan(0);
    expect(screen.getAllByText('3m').length).toBeGreaterThan(0);
    expect(screen.getAllByText('5.0 MB').length).toBeGreaterThan(0);
    expect(screen.getAllByText('@drizztdourden08/tessera').length).toBeGreaterThan(0);
    expect(screen.getAllByText('2.0 KB').length).toBeGreaterThan(0);
  });

  it('lists only the consumers that are switched on, with their packages', async () => {
    open(REPORT);
    const table = within(await screen.findByRole('table', { name: 'Consumers switched on' }));
    expect(table.getByText('X:/rotp')).toBeTruthy();
    expect(table.getByText('@drizztdourden08/tessera')).toBeTruthy();
    expect(table.queryByText('X:/bxel')).toBeNull();
  });

  it('warns when the GitHub scope points at the local store', async () => {
    open({ ...REPORT, scopeRedirected: true });
    expect(await screen.findByText('The GitHub package scope points at the local store (127.0.0.1:9) while a consumer is on.')).toBeTruthy();
  });

  it('has no warning when the scope is not redirected', async () => {
    open(REPORT);
    await screen.findAllByText('X:/brock');
    expect(screen.queryByText(/points at the local store/)).toBeNull();
  });

  it('says when the registry could not be read', async () => {
    stubRemote({});
    renderAt('/#registry');
    expect(await screen.findByText(/The local registry could not be read/)).toBeTruthy();
  });
});
