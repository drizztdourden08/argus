/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DEPS, dep } from './helpers/deps';
import { openOverview } from './helpers/open-overview';
import { setupOverview } from './helpers/setup-overview';

setupOverview();

const open = (deps: Record<string, object> = DEPS, location = '/#dependencies') => openOverview(location, deps);

const linksOf = (): string[] => within(screen.getByRole('list', { name: 'Dependencies between projects' })).getAllByRole('link').map((link) => link.textContent);

describe('S1 the Dependencies tab', { tags: ['standard'] }, () => {
  it('lists the open requests, longest wait first, with the item and the other item as links', async () => {
    await open();
    await screen.findByRole('list', { name: 'Dependencies between projects' });
    expect(linksOf()).toEqual(['archipelia/I-7', 'rotp/I-12', 'argus/I-3', 'brock/I-4']);
    expect(screen.getAllByRole('link', { name: 'rotp/I-12' })[0]?.getAttribute('href')).toBe('/p/rotp?open=I-12');
  });

  it('shows the blocked requests on Blocked and the done ones on Resolved', async () => {
    await open();
    await screen.findByRole('list', { name: 'Dependencies between projects' });
    fireEvent.click(screen.getByRole('radio', { name: 'Blocked' }));
    await waitFor(() => expect(linksOf()).toEqual(['rotp/I-2', 'argus/I-9']));
    fireEvent.click(screen.getByRole('radio', { name: 'Resolved' }));
    await waitFor(() => expect(linksOf()).toEqual(['brock/I-1', 'argus/I-5']));
    fireEvent.click(screen.getByRole('radio', { name: 'All' }));
    await waitFor(() => expect(linksOf()).toHaveLength(8));
  });

  it('says so when no request matches the filter', async () => {
    await open({ 'D-4': DEPS['D-4'] as object });
    expect(await screen.findByText('No request is waiting on another project.')).toBeTruthy();
    fireEvent.click(screen.getByRole('radio', { name: 'Resolved' }));
    expect(await screen.findByRole('list', { name: 'Dependencies between projects' })).toBeTruthy();
  });

  it('counts the blocked requests and the open ones over 3 days on the tab', async () => {
    await open(DEPS, '/');
    await waitFor(() => expect(screen.getByRole('tab', { name: /Dependencies/ }).textContent).toContain('2'));
  });
});

describe('I-85 P7 the dependency rows', { tags: ['critical'] }, () => {
  it('draws each request as a dependency row with the DP code and no trail chip for a single link', async () => {
    await open();
    const list = await screen.findByRole('list', { name: 'Dependencies between projects' });
    const rows = [...list.querySelectorAll<HTMLElement>('[data-record]')];
    expect(rows.map((row) => row.dataset.record)).toEqual(['DP-1', 'DP-2']);
    expect(within(rows[0] as HTMLElement).queryByText(/^trail/)).toBeNull();
  });

  it('shows a trail chip when the request waits on an item that waits on another', async () => {
    const chain = { 'D-1': DEPS['D-1'] as object, 'D-5': dep('D-5', { from: { project: 'rotp', item: 'I-12', thread: null }, to: { project: 'brock', item: 'I-8', thread: null } }) };
    await open(chain);
    const list = await screen.findByRole('list', { name: 'Dependencies between projects' });
    const first = list.querySelector<HTMLElement>('[data-record="DP-1"]');
    expect(within(first as HTMLElement).getByText('trail 2')).toBeTruthy();
  });
});
