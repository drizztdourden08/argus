/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { filterBoard, stored, typeSearch } from './filter-dom';
import { buildDoc, doneDoc, itemDoc, testDoc, threadDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const groupOf = (container: HTMLElement, key: string): HTMLElement => container.querySelector<HTMLElement>(`[data-group="${key}"]`) as HTMLElement;

const titlesIn = (card: HTMLElement): string[] => [...card.querySelectorAll('[data-test]')].map((row) => row.getAttribute('data-test') ?? '');

describe('grouping', () => {
  it('by item makes one card per item with its tests and no item chip', async () => {
    const { container } = await mountTab(filterBoard());
    await screen.findByText('Login works');
    fireEvent.click(screen.getByRole('radio', { name: 'Item' }));
    expect(titlesIn(groupOf(container, 'I-1'))).toEqual(['T-1', 'T-3']);
    expect(titlesIn(groupOf(container, 'I-2'))).toEqual(['T-2']);
    expect(within(groupOf(container, 'I-1')).getByText('Item I-1')).toBeTruthy();
    expect(localStorage.getItem('coord.g.tests3')).toBe('item');
  });

  it('by thread follows the item and names the thread by its id', async () => {
    const board = { ...filterBoard(), threads: [threadDoc('lead', { session: 'Overlord' }), threadDoc('coder')] };
    const { container } = await mountTab(board);
    await screen.findByText('Login works');
    fireEvent.click(screen.getByRole('radio', { name: 'Thread' }));
    expect(titlesIn(groupOf(container, 'coder'))).toEqual(['T-2']);
    expect(within(groupOf(container, 'lead')).getAllByText('lead').length).toBeGreaterThan(0);
  });

  it('returns to the bundles and reads the saved choice on load', async () => {
    localStorage.setItem('coord.g.tests3', 'item');
    const { container } = await mountTab(filterBoard());
    await screen.findByText('Login works');
    expect(groupOf(container, 'I-1')).not.toBeNull();
    fireEvent.click(screen.getByRole('radio', { name: 'Bundle' }));
    expect(container.querySelector('[data-bundle="b1"]')).not.toBeNull();
    expect(localStorage.getItem('coord.g.tests3')).toBe('build');
  });

  it('says there is no test at all when the board has none', async () => {
    await mountTab({});
    expect(await screen.findByText('No test yet. Threads document every test here with coord test.')).toBeTruthy();
  });
});

describe('the order', () => {
  const ranked = () => ({
    builds: [buildDoc('b1')],
    tests: [
      testDoc('T-1', { rank: 2, updatedAt: '2026-10-26T09:30:00.000Z' }),
      testDoc('T-2', { rank: 1, updatedAt: '2026-10-26T09:10:00.000Z' }),
    ],
  });
  const order = (container: HTMLElement): string[] => titlesIn(container);

  it('is the manual rank until the owner picks updated, and is saved with the filters', async () => {
    const { container } = await mountTab(ranked());
    await screen.findByText('Test T-1');
    expect(order(container)).toEqual(['T-2', 'T-1']);
    fireEvent.click(screen.getByRole('radio', { name: 'Updated' }));
    expect(order(container)).toEqual(['T-1', 'T-2']);
    expect(stored('coord.f')).toMatchObject({ sort: 'updated' });
  });
});

describe('show passed', () => {
  const withPassed = () => ({ builds: [buildDoc('b1')], tests: [testDoc('T-1'), doneDoc('T-2', 'pass', 'fine'), doneDoc('T-3', 'fail', 'broken')] });

  it('is on by default, hides the passed tests when turned off and remembers it', async () => {
    await mountTab(withPassed());
    await screen.findByText('Test T-2');
    fireEvent.click(screen.getByRole('button', { name: 'Hide passed' }));
    expect(screen.queryByText('Test T-2')).toBeNull();
    expect(screen.getByText('Test T-3')).toBeTruthy();
    expect(localStorage.getItem('coord.passed')).toBe('0');
  });

  it('reads the saved choice on load', async () => {
    localStorage.setItem('coord.passed', '0');
    await mountTab(withPassed());
    await screen.findByText('Test T-3');
    expect(screen.queryByText('Test T-2')).toBeNull();
  });
});

describe('closed bundles and the search', () => {
  const closed = () => ({
    builds: [buildDoc('open-1'), buildDoc('old-1', { closedAt: '2026-10-25T10:00:00.000Z', closedBy: 'lead' })],
    items: [itemDoc('I-1')],
    tests: [testDoc('T-1', { build: 'open-1' }), testDoc('T-4', { build: 'old-1' })],
  });

  it('are never reached by a search unless Show closed is on', async () => {
    const off = await mountTab(closed());
    await screen.findByText('Test T-1');
    typeSearch('T-4');
    expect(off.container.querySelector('[data-bundle="old-1"]')).toBeNull();
    off.unmount();
    localStorage.clear();
    localStorage.setItem('coord.closed', '1');
    const on = await mountTab(closed());
    await screen.findByText('Test T-1');
    typeSearch('T-4');
    expect(on.container.querySelector('[data-bundle="old-1"]')).not.toBeNull();
    expect(on.container.querySelector('[data-bundle="open-1"]')).toBeNull();
  });

  it('remember the Show closed choice under the key of the Node page', async () => {
    await mountTab(closed());
    fireEvent.click(await screen.findByRole('button', { name: 'Show closed (1)' }));
    expect(localStorage.getItem('coord.closed')).toBe('1');
    fireEvent.click(screen.getByRole('button', { name: 'Hide closed (1)' }));
    expect(localStorage.getItem('coord.closed')).toBe('0');
  });
});
