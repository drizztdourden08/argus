/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { addFlag, openFilters, pickClause } from '../helpers/filter-clauses';
import { filterBoard, shown, stored, typeSearch } from './filter-dom';
import { mountTab } from './mount-tab';

cleanAfterEach();

describe('S1 search', { tags: ['standard'] }, () => {
  it('narrows the tests by id or title and says how many are shown', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await openFilters();
    typeSearch('logout');
    shown('T-2');
    expect(screen.getByText('2 to run / 3 · filtered')).toBeTruthy();
    typeSearch('t-3');
    shown('T-3');
  });

  it('keeps declared bundles out while a filter is on, and brings them back when the search is emptied', async () => {
    const { container } = await mountTab({ ...filterBoard(), builds: [...filterBoard().builds, { ...filterBoard().builds[0], tag: 'b3', id: 'b3' }] });
    await screen.findByText('Login works');
    await openFilters();
    expect(container.querySelector('[data-bundle="b3"]')).not.toBeNull();
    typeSearch('login');
    expect(container.querySelector('[data-bundle="b3"]')).toBeNull();
    typeSearch('');
    expect(container.querySelector('[data-bundle="b3"]')).not.toBeNull();
  });
});

describe('S1 the other filters', { tags: ['standard'] }, () => {
  it('filter by status, by subject and by what needs you', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await openFilters();
    await pickClause('Status', 'Failed');
    shown('T-3');
    fireEvent.click(screen.getByRole('button', { name: 'Remove filter on Status' }));
    await pickClause('Subject', 'play');
    shown('T-2');
    fireEvent.click(screen.getByRole('button', { name: 'Remove filter on Subject' }));
    await addFlag('Needs you');
    shown('T-1');
  });

  it('filter by thread, which a test takes from its item', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await openFilters();
    await pickClause('Threads', 'coder');
    shown('T-2');
  });

  it('filter by build, with no build as a choice of its own', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await openFilters();
    await pickClause('Build', 'b2');
    shown('T-2');
    await pickClause('Build', 'no build');
    shown('T-2', 'T-3');
  });
});

describe('S1 the saved filters', { tags: ['standard'] }, () => {
  it('are written under the keys of the Node page and read back on load', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await openFilters();
    typeSearch('works');
    await addFlag('Archived');
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
