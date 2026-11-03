/* @layer renderer-app @kind test */
import { render, screen, within } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { ReleasesTab } from '../../src/screens/board/tabs/releases';
import { cleanAfterEach } from '../board-tests/board-dom';
import { createFakeSources } from '../helpers/fake-source';
import { renderAt } from '../helpers/render-at';
import { ago, MIN, replay, versionDoc } from './helpers/fixtures';
import { stubRemote } from './helpers/stub-remote';

cleanAfterEach();

const CHANNELS = [
  {
    channel: 'stable',
    latest: '1.2.0',
    versions: [
      versionDoc('1.1.0', { publishedAt: ago(60 * MIN), replay: replay('done', 'https://github.com/o/r/releases/tag/v1.1.0') }),
      versionDoc('1.2.0', { publishedAt: ago(5 * MIN) }),
    ],
  },
  { channel: 'beta', latest: '1.3.0-beta.1', versions: [versionDoc('1.3.0-beta.1', { notesUrl: null, assets: [], packages: [] })] },
];

const mountBoard = (routes: Record<string, unknown>) => {
  stubRemote(routes);
  const fake = createFakeSources();
  return render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter>
        <ReleasesTab boardKey="rotp" />
      </MemoryRouter>
    </DataProvider>,
  );
};

describe('S1 the board Releases tab', { tags: ['standard'] }, () => {
  it('shows the channels and marks the latest version', async () => {
    mountBoard({ '/api/p/rotp/releases': { channels: CHANNELS } });
    expect(await screen.findByRole('group', { name: 'Channel stable' })).toBeTruthy();
    expect(screen.getByText('latest 1.2.0')).toBeTruthy();
    const newest = within(screen.getByRole('group', { name: 'Release 1.2.0 of channel stable' }));
    expect(newest.getByText('latest')).toBeTruthy();
    const older = within(screen.getByRole('group', { name: 'Release 1.1.0 of channel stable' }));
    expect(older.queryByText('latest')).toBeNull();
  });

  it('shows the title, tag, short commit, age, notes link and replay state of a version', async () => {
    mountBoard({ '/api/p/rotp/releases': { channels: CHANNELS } });
    const card = within(await screen.findByRole('group', { name: 'Release 1.2.0 of channel stable' }));
    expect(card.getByText('Release 1.2.0')).toBeTruthy();
    expect(card.getByText('v1.2.0')).toBeTruthy();
    expect(card.getByText('0123456')).toBeTruthy();
    expect(card.getByText('5m ago')).toBeTruthy();
    expect(card.getByRole('link', { name: 'Release notes' }).getAttribute('href')).toBe('/releases/rotp/stable/1.2.0/notes.md');
    expect(card.getByText('waits for the go')).toBeTruthy();
    const done = within(screen.getByRole('group', { name: 'Release 1.1.0 of channel stable' }));
    expect(done.getByRole('link', { name: 'on GitHub' }).getAttribute('href')).toBe('https://github.com/o/r/releases/tag/v1.1.0');
  });

  it('counts the files and packages of a version on its card and leaves the lists to the Files tab', async () => {
    mountBoard({ '/api/p/rotp/releases': { channels: CHANNELS } });
    const card = within(await screen.findByRole('group', { name: 'Release 1.2.0 of channel stable' }));
    expect(card.getByText('1 package')).toBeTruthy();
    expect(card.getByText('1 file')).toBeTruthy();
    expect(card.queryByRole('link', { name: 'rotp-1.2.0-setup.exe' })).toBeNull();
    expect(card.getByRole('button', { name: /^Open V-1\.2\.0/ })).toBeTruthy();
  });

  it('says when the releases could not be read', async () => {
    mountBoard({});
    expect(await screen.findByText(/The releases could not be read/)).toBeTruthy();
  });
});

describe('S1 the overview Releases tab', { tags: ['standard'] }, () => {
  it('groups the releases by project', async () => {
    stubRemote({
      '/api/releases': {
        projects: [
          { project: 'rotp', channels: CHANNELS },
          { project: 'brock', channels: [{ channel: 'stable', latest: '0.9.0', versions: [versionDoc('0.9.0')] }] },
          { project: 'empty', channels: [] },
        ],
      },
    });
    renderAt('/#releases');
    expect(await screen.findByRole('link', { name: 'rotp' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'brock' })).toBeTruthy();
    expect(screen.queryByRole('link', { name: 'empty' })).toBeNull();
    expect(screen.getByRole('group', { name: 'Release 0.9.0 of channel stable' })).toBeTruthy();
    expect(screen.getByRole('group', { name: 'Release 1.2.0 of channel stable' })).toBeTruthy();
  });
});
