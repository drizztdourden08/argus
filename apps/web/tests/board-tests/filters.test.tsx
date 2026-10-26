/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { filterBoard, pick, stored, typeSearch } from './filter-dom';
import { mountTab } from './mount-tab';

cleanAfterEach();

const TITLES: Record<string, string> = { 'T-1': 'Login works', 'T-2': 'Logout works', 'T-3': 'Sound plays' };

const shown = (...ids: string[]): void => {
  for (const [id, title] of Object.entries(TITLES)) expect(screen.queryByText(title) !== null).toBe(ids.includes(id));
};

describe('search', () => {
  it('narrows the tests by id or title and says how many are shown', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    typeSearch('logout');
    shown('T-2');
    expect(screen.getByText('2 to run / 3 · filtered')).toBeTruthy();
    typeSearch('t-3');
    shown('T-3');
  });

  it('says so when no test matches', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    typeSearch('zzz');
    expect(screen.getByText('No test matches the filters.')).toBeTruthy();
  });

  it('keeps declared bundles out while a filter is on, and clears with Clear filters', async () => {
    const { container } = await mountTab({ ...filterBoard(), builds: [...filterBoard().builds, { ...filterBoard().builds[0], tag: 'b3', id: 'b3' }] });
    await screen.findByText('Login works');
    expect(container.querySelector('[data-bundle="b3"]')).not.toBeNull();
    typeSearch('login');
    expect(container.querySelector('[data-bundle="b3"]')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Clear filters' }));
    expect(container.querySelector('[data-bundle="b3"]')).not.toBeNull();
  });
});

describe('the other filters', () => {
  it('filter by status, by subject and by what needs you', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await pick('Status', 'Failed');
    shown('T-3');
    fireEvent.click(screen.getByRole('button', { name: 'Clear filters' }));
    await pick('Subject', 'play');
    shown('T-2');
    fireEvent.click(screen.getByRole('button', { name: 'Clear filters' }));
    fireEvent.click(screen.getByRole('switch', { name: 'needs you' }));
    shown('T-1');
  });

  it('filter by thread, which a test takes from its item', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await pick('Threads', 'coder');
    shown('T-2');
  });

  it('filter by build, with no build as a choice of its own', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await pick('Build', 'b2');
    shown('T-2');
    await pick('Build', 'no build');
    shown('T-2', 'T-3');
  });
});

describe('the saved filters', () => {
  it('are written under the keys of the Node page and read back on load', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    typeSearch('works');
    fireEvent.click(screen.getByRole('switch', { name: 'archived' }));
    expect(stored('coord.f')).toMatchObject({ q: 'works', archived: true, sort: 'rank', builds: [], nobuild: false });
  });

  it('keep the keys another tab wrote', async () => {
    localStorage.setItem('coord.f', JSON.stringify({ trash: true, q: 'logout', conf: false }));
    await mountTab(filterBoard());
    await screen.findByText('Logout works');
    shown('T-2');
    expect(stored('coord.f')).toMatchObject({ trash: true, q: 'logout' });
  });
});
