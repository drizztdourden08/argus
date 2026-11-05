/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { openFilters } from '../helpers/filter-clauses';
import { cleanAfterEach } from './board-dom';
import { filterBoard, pick } from './filter-dom';
import { threadDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

describe('S1 the focus row', { tags: ['standard'] }, () => {
  it('narrows the tests to one thread, and All threads brings them back', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await openFilters();
    await pick('Focus', 'coder');
    expect(screen.queryByText('Login works')).toBeNull();
    expect(screen.getByText('Logout works')).toBeTruthy();
    expect(localStorage.getItem('coord.focus.rotp')).toBe('coder');
    expect(screen.getByRole('img', { name: '1 active' })).toBeTruthy();
    await pick('Focus', 'All threads');
    expect(await screen.findByText('Login works')).toBeTruthy();
    expect(localStorage.getItem('coord.focus.rotp')).toBe('');
  });

  it('hides the tests of a paused thread until Show paused is on', async () => {
    const board = { ...filterBoard(), threads: [threadDoc('lead'), threadDoc('coder', { status: 'paused' })] };
    await mountTab(board);
    await screen.findByText('Login works');
    await openFilters();
    expect(screen.queryByText('Logout works')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show paused (1)' }));
    expect(await screen.findByText('Logout works')).toBeTruthy();
    expect(localStorage.getItem('coord.paused')).toBe('1');
  });
});
