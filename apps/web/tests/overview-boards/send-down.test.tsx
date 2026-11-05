/* @layer renderer-app @kind test */
import { fireEvent, render, screen, within } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { useRunningRuns } from '../../src/screens/overview/boards/behavior/useRunningRuns';
import { RunningNow } from '../../src/screens/overview/boards/sub-components/RunningNow';
import { createFakeSources } from '../helpers/fake-source';
import { minutesAgo } from '../overview/helpers/fixtures';
import { installHarness } from '../overview/helpers/harness';
import { stubApi } from '../overview/helpers/stub-api';
import type { ApiCall } from '../overview/helpers/stub-api';
import { COUNTS, item } from './helpers';
import { renderCard } from './render-card';

installHarness();

const posts = (calls: ApiCall[]): ApiCall[] => calls.filter((call) => call.method === 'POST');

const NowHost = () => <RunningNow runs={useRunningRuns()} />;

const renderNow = () => {
  const fake = createFakeSources();
  return render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter>
        <NowHost />
      </MemoryRouter>
    </DataProvider>,
  );
};

describe('P7 the send down limit', { tags: ['standard'] }, () => {
  it('refuses a text over 50000 characters with a plain sentence and posts nothing', async () => {
    const calls = stubApi();
    renderCard(item('rotp', { name: 'RotP', counts: { ...COUNTS, threads: 1 } }));
    fireEvent.click(await screen.findByRole('button', { name: 'Send to all' }));
    const dialog = await screen.findByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Text to send down'), { target: { value: 'x'.repeat(50001) } });
    expect(await within(dialog).findByText(/This text is 50001 characters and one post takes 50000 at most/)).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Send down' }).hasAttribute('disabled')).toBe(true);
    expect(posts(calls)).toEqual([]);
  });
});

describe('P7 running now', { tags: ['standard'] }, () => {
  it('lists the live runs of every project', async () => {
    stubApi((call) => (call.path.startsWith('/api/runs')
      ? { body: { runs: [{ id: 'R-5', scope: 'rotp', name: 'hello', status: 'running', queuedAt: minutesAgo(1), startedAt: minutesAgo(1), lastLine: 'compiling' }] } }
      : undefined));
    renderNow();
    expect(await screen.findByText('hello R-5')).toBeTruthy();
    expect(screen.getByText('compiling')).toBeTruthy();
  });
});
