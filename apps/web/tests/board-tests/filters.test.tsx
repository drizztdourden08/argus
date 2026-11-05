/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { filterBoard, pick, shown, stored, typeSearch } from './filter-dom';
import { mountTab } from './mount-tab';

cleanAfterEach();

describe('S1 search', { tags: ['standard'] }, () => {
  it('narrows the tests by id or title and says how many are shown', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    typeSearch('logout');
    shown('T-2');
    expect(screen.getByText('2 to run / 3 · filtered')).toBeTruthy();
    typeSearch('t-3');
    shown('T-3');
  });

});

describe('S1 the other filters', { tags: ['standard'] }, () => {
  it('filter by status, by subject and by what needs you', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await pick('Status', 'Failed');
    shown('T-3');
    fireEvent.click(screen.getByRole('button', { name: 'Clear filters' }));
    await pick('Subject', 'play');
    shown('T-2');
    fireEvent.click(screen.getByRole('button', { name: 'Clear filters' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'needs you' }));
    shown('T-1');
  });

  it('filter by thread, which a test takes from its item', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await pick('Threads', 'coder');
    shown('T-2');
  });
});

describe('S1 the saved filters', { tags: ['standard'] }, () => {
  it('are written under the keys of the Node page and read back on load', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    typeSearch('works');
    fireEvent.click(screen.getByRole('checkbox', { name: 'archived' }));
    expect(stored('coord.f')).toMatchObject({ q: 'works', archived: true, sort: 'rank', });
  });

  it('keep the keys another tab wrote', async () => {
    localStorage.setItem('coord.f', JSON.stringify({ trash: true, q: 'logout', conf: false }));
    await mountTab(filterBoard());
    await screen.findByText('Logout works');
    shown('T-2');
    expect(stored('coord.f')).toMatchObject({ trash: true, q: 'logout' });
  });
});
