/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { ACTIVITY, CARDS } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';

installHarness();
beforeEach(() => {
  stubApi();
});

const open = async (activity: Record<string, object> = ACTIVITY) => {
  const view = renderAt('/#activity');
  await feed(view.sources, { cards: CARDS, activity });
  return view;
};

describe('the Activity tab', () => {
  it('says there is nothing to report when the feed is empty', async () => {
    await open({});
    expect(await screen.findByText('Nothing to report yet.')).toBeTruthy();
  });

  it('shows each entry with the time ago, its text, and the level, project and thread', async () => {
    await open();
    expect(await screen.findByText('R-3 play failed (exit 1)')).toBeTruthy();
    expect(screen.getByText('ping from rotp/lead: build ready')).toBeTruthy();
    expect(screen.getByText('error RotP / lead')).toBeTruthy();
    expect(screen.getByText('warn RotP / lead')).toBeTruthy();
    expect(screen.getAllByText(/^\d+m ago$/).length).toBeGreaterThanOrEqual(3);
  });

  it('filters by level with the counts on the buttons, and the all button clears it', async () => {
    await open();
    await screen.findByText('R-3 play failed (exit 1)');
    expect(screen.getByText('All levels 3')).toBeTruthy();
    fireEvent.click(screen.getByText('error 1'));
    await waitFor(() => expect(screen.queryByText('ping from rotp/lead: build ready')).toBeNull());
    expect(screen.getByText('R-3 play failed (exit 1)')).toBeTruthy();
    fireEvent.click(screen.getByText('All levels 3'));
    expect(await screen.findByText('ping from rotp/lead: build ready')).toBeTruthy();
  });

  it('says nothing matches when a level has no entries', async () => {
    await open({ 'a-2': ACTIVITY['a-2'] });
    await screen.findByText('ping from rotp/lead: build ready');
    fireEvent.click(screen.getByText('warn 0'));
    expect(await screen.findByText('Nothing matches these filters.')).toBeTruthy();
  });

  it('shows a new entry as it arrives on the stream', async () => {
    const view = await open({ 'a-2': ACTIVITY['a-2'] });
    await screen.findByText('ping from rotp/lead: build ready');
    await feed(view.sources, { activity: { 'a-2': ACTIVITY['a-2'], 'a-9': { n: 9, at: new Date().toISOString(), kind: 'wake', level: 'info', text: 'woke rotp/lead' } } });
    expect(await screen.findByText('woke rotp/lead')).toBeTruthy();
  });

  it('shows a doc with no time, no kind, no level and no text as a row', async () => {
    await open({ 'a-7': { n: 7 } });
    expect(await screen.findByText('All levels 1')).toBeTruthy();
  });
});
