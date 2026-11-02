/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { DEPS } from './helpers/deps';
import { feed } from './helpers/feed';
import { CARDS } from './helpers/fixtures';
import { installHarness } from './helpers/harness';
import { stubApi } from './helpers/stub-api';

installHarness();
beforeEach(() => {
  stubApi();
});

const open = async (deps: Record<string, object> = DEPS, location = '/#dependencies') => {
  const view = renderAt(location);
  await feed(view.sources, { cards: CARDS, deps });
  return view;
};

const linksOf = (): string[] => within(screen.getByRole('table', { name: 'Dependencies between projects' })).getAllByRole('link').map((link) => link.textContent ?? '');

describe('S1 the Dependencies tab', { tags: ['standard'] }, () => {
  it('lists the open requests, longest wait first, with the item and the other item as links', async () => {
    await open();
    await screen.findByRole('table', { name: 'Dependencies between projects' });
    expect(linksOf()).toEqual(['archipelia/I-7', 'rotp/I-12', 'argus/I-3', 'brock/I-4']);
    expect(screen.getAllByRole('link', { name: 'rotp/I-12' })[0]?.getAttribute('href')).toBe('/p/rotp#I-12');
  });

  it('shows the blocked requests on Blocked and the done ones on Resolved', async () => {
    await open();
    await screen.findByRole('table', { name: 'Dependencies between projects' });
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
    expect(await screen.findByRole('table', { name: 'Dependencies between projects' })).toBeTruthy();
  });

  it('counts the blocked requests and the open ones over 3 days on the tab', async () => {
    await open(DEPS, '/');
    await waitFor(() => expect(screen.getByRole('tab', { name: /Dependencies/ }).textContent).toContain('2'));
  });
});
