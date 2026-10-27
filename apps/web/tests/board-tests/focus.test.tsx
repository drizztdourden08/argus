/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { filterBoard, pick } from './filter-dom';
import { threadDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

describe('the focus row', () => {
  it('narrows the tests to one thread and says so, and Show all brings them back', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await pick('Focus', 'coder');
    expect(screen.queryByText('Login works')).toBeNull();
    expect(screen.getByText('Logout works')).toBeTruthy();
    expect(screen.getByText('Only coder is shown. Its open questions are in Activity.')).toBeTruthy();
    expect(localStorage.getItem('coord.focus')).toBe('coder');
    fireEvent.click(screen.getByRole('button', { name: 'Show all' }));
    expect(await screen.findByText('Login works')).toBeTruthy();
  });

  it('hides the tests of a paused thread until Show paused is on', async () => {
    const board = { ...filterBoard(), threads: [threadDoc('lead'), threadDoc('coder', { status: 'paused' })] };
    await mountTab(board);
    await screen.findByText('Login works');
    expect(screen.queryByText('Logout works')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show paused (1)' }));
    expect(await screen.findByText('Logout works')).toBeTruthy();
    expect(localStorage.getItem('coord.paused')).toBe('1');
  });
});
