/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const board = (at?: string) => ({
  at,
  builds: [buildDoc('bundle-1')],
  tests: [testDoc('T-1', { rank: 1 }), testDoc('T-2', { rank: 2 }), testDoc('T-3', { rank: 3 })],
});

const hash = (): string => screen.getByTestId('hash').textContent;

const drawer = (): HTMLElement => screen.getByRole('dialog', { name: 'Test details' });

describe('the anchor of a test', () => {
  it('opens the drawer of the test named in the address', async () => {
    await mountTab(board('/#T-2'));
    expect(await screen.findByRole('heading', { name: 'Test T-2' })).toBeTruthy();
    expect(hash()).toBe('#T-2');
  });

  it('is written to the address when a row opens, and gives way to the tab when the drawer closes', async () => {
    await mountTab(board('/#tests'));
    fireEvent.click(await screen.findByRole('button', { name: 'Test T-3' }));
    expect(hash()).toBe('#T-3');
    expect(await screen.findByRole('heading', { name: 'Test T-3' })).toBeTruthy();
    fireEvent.keyDown(drawer(), { key: 'Escape' });
    await waitFor(() => expect(hash()).toBe('#tests'));
  });

  it('opens nothing for an address that names no test', async () => {
    await mountTab(board('/#T-99'));
    await screen.findByText('Test T-1');
    expect(screen.queryByRole('heading', { name: /^Test T-/ })).toBeNull();
  });

  it('copies the address with the anchor', async () => {
    const writeText = vi.fn(() => Promise.resolve());
    vi.stubGlobal('navigator', { clipboard: { writeText } });
    await mountTab(board('/#T-1'));
    fireEvent.click(await screen.findByRole('button', { name: 'Copy a link to T-1' }));
    expect(writeText).toHaveBeenCalledWith(`${window.location.origin}${window.location.pathname}#T-1`);
  });
});

describe('walking the list with [ and ]', () => {
  it('moves to the next and the previous test of the list the drawer was opened from', async () => {
    await mountTab(board());
    fireEvent.click(await screen.findByRole('button', { name: 'Test T-1' }));
    const previous = screen.getByRole<HTMLButtonElement>('button', { name: 'Previous in the list ([)' });
    expect(previous.disabled).toBe(true);
    fireEvent.keyDown(document.body, { key: ']' });
    expect(hash()).toBe('#T-2');
    fireEvent.keyDown(document.body, { key: ']' });
    expect(hash()).toBe('#T-3');
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Next in the list (])' }).disabled).toBe(true);
    fireEvent.keyDown(document.body, { key: ']' });
    expect(hash()).toBe('#T-3');
    fireEvent.click(screen.getByRole('button', { name: 'Previous in the list ([)' }));
    expect(hash()).toBe('#T-2');
    fireEvent.keyDown(document.body, { key: '[' });
    expect(hash()).toBe('#T-1');
  });

  it('leaves the brackets to a field that is being typed in, and to a held modifier', async () => {
    await mountTab(board());
    fireEvent.click(await screen.findByRole('button', { name: 'Test T-1' }));
    fireEvent.keyDown(within(drawer()).getByRole('textbox', { name: 'Caption for T-1' }), { key: ']' });
    fireEvent.keyDown(document.body, { key: ']', ctrlKey: true });
    expect(hash()).toBe('#T-1');
  });
});

describe('the focus mode', () => {
  it('widens the drawer to the whole width and remembers it', async () => {
    await mountTab(board('/#T-1'));
    const toggle = await screen.findByRole('button', { name: 'Expand' });
    expect(document.querySelector('.drawer--xl')).toBeNull();
    fireEvent.click(toggle);
    expect(localStorage.getItem('coord.drawer.wide')).toBe('1');
    expect(document.querySelector('.drawer--xl')).not.toBeNull();
    expect(document.querySelector('.drawer--top')).toBeNull();
    expect(screen.getByRole('button', { name: 'Collapse' }).getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(screen.getByRole('button', { name: 'Collapse' }));
    expect(localStorage.getItem('coord.drawer.wide')).toBe('0');
  });

  it('reads the saved choice on load', async () => {
    localStorage.setItem('coord.drawer.wide', '1');
    await mountTab(board('/#T-1'));
    expect(await screen.findByRole('button', { name: 'Collapse' })).toBeTruthy();
    expect(drawer().textContent).toContain('T-1');
  });
});
