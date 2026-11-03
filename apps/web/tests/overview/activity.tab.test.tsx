/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { feed } from './helpers/feed';
import { ACTIVITY, CARDS, minutesAgo } from './helpers/fixtures';
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

const rowsOf = (): HTMLElement[] => within(screen.getByRole('table', { name: 'Activity' })).getAllByRole('row');

describe('S1 the Activity tab', { tags: ['standard'] }, () => {
  it('shows the kind, the level, the time and the project and thread of each entry', async () => {
    await open();
    await screen.findByText('R-3 play failed (exit 1)');
    const first = rowsOf()[2] as HTMLElement;
    expect(within(first).getByText('run')).toBeTruthy();
    expect(within(first).getByText('/ lead')).toBeTruthy();
    expect(within(first).getByRole('link', { name: 'RotP' }).getAttribute('href')).toBe('/p/rotp/');
    expect(first.textContent).toMatch(/\d+[smhd]\sago/);
  });
});

describe('S1 the filters of the Activity tab', { tags: ['standard'] }, () => {
  it('filters by level with the counts on the buttons, and the all button clears it', async () => {
    await open();
    await screen.findByText('R-3 play failed (exit 1)');
    expect(screen.getByText('All levels 3')).toBeTruthy();
    fireEvent.click(screen.getByText('error 1'));
    await waitFor(() => expect(rowsOf()).toHaveLength(1));
    expect(screen.getByText('R-3 play failed (exit 1)')).toBeTruthy();
    fireEvent.click(screen.getByText('All levels 3'));
    await waitFor(() => expect(rowsOf()).toHaveLength(3));
  });

  it('pages in steps of 50, says how many are shown, and shows the rest on Show more', async () => {
    const many = Object.fromEntries(Array.from({ length: 60 }, (_, index) => [`a-${index + 1}`, { n: index + 1, at: minutesAgo(index + 1), kind: 'note', level: 'info', text: `entry ${index + 1}` }]));
    await open(many);
    await screen.findByText('50 of 60 shown');
    expect(rowsOf()).toHaveLength(50);
    fireEvent.click(screen.getByRole('button', { name: 'Show more' }));
    await waitFor(() => expect(rowsOf()).toHaveLength(60));
    expect(screen.queryByRole('button', { name: 'Show more' })).toBeNull();
  });

  it('filters by kind with a chip per kind and its count, the pressed chip again clears it', async () => {
    await open();
    await screen.findByText('R-3 play failed (exit 1)');
    expect(screen.getByRole('radio', { name: 'All kinds 3' }).getAttribute('aria-checked')).toBe('true');
    expect(screen.getByRole('radio', { name: 'kill 0' })).toBeTruthy();
    fireEvent.click(screen.getByRole('radio', { name: 'fault 1' }));
    await waitFor(() => expect(rowsOf()).toHaveLength(1));
    expect(rowsOf()[0]?.textContent).toContain('its wake listener is not running');
    expect(screen.getByRole('radio', { name: 'fault 1' }).getAttribute('aria-checked')).toBe('true');
    fireEvent.click(screen.getByRole('radio', { name: 'fault 1' }));
    await waitFor(() => expect(rowsOf()).toHaveLength(3));
    expect(screen.getByRole('radio', { name: 'All kinds 3' }).getAttribute('aria-checked')).toBe('true');
  });

  it('combines the level and the kind filters', async () => {
    await open();
    await screen.findByText('R-3 play failed (exit 1)');
    fireEvent.click(screen.getByText('error 1'));
    fireEvent.click(screen.getByRole('radio', { name: 'fault 1' }));
    expect(await screen.findByText('Nothing matches these filters.')).toBeTruthy();
  });
});

describe('S1 the Activity tab as it changes', { tags: ['standard'] }, () => {
  it('shows a new entry at the top as it arrives on the stream', async () => {
    const view = await open({ 'a-2': ACTIVITY['a-2'] });
    await screen.findByText('ping from rotp/lead: build ready');
    await feed(view.sources, { activity: { 'a-2': ACTIVITY['a-2'], 'a-9': { n: 9, at: minutesAgo(0), kind: 'wake', level: 'info', text: 'woke rotp/lead' } } });
    await screen.findByText('woke rotp/lead');
    expect(rowsOf()[0]?.textContent).toContain('woke rotp/lead');
  });
});
