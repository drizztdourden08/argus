/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { openFilters, pickView } from '../helpers/filter-clauses';
import { filterBoard, stored, typeSearch } from './filter-dom';
import { doneDoc, itemDoc, testDoc, testRunDoc, threadDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const groupOf = (container: HTMLElement, key: string): HTMLElement => container.querySelector<HTMLElement>(`[data-group="${key}"]`) as HTMLElement;

const titlesIn = (card: HTMLElement): string[] => [...card.querySelectorAll('[data-test]')].map((row) => row.getAttribute('data-test') ?? '');

describe('S1 grouping', { tags: ['standard'] }, () => {
  it('by item makes one card per item with its tests and no item chip', async () => {
    const { container } = await mountTab(filterBoard());
    await screen.findByText('Login works');
    await pickView(/^Group/, 'Item');
    expect(titlesIn(groupOf(container, 'I-1'))).toEqual(['T-1', 'T-3']);
    expect(titlesIn(groupOf(container, 'I-2'))).toEqual(['T-2']);
    expect(within(groupOf(container, 'I-1')).getByText('Item number 1')).toBeTruthy();
    expect(localStorage.getItem('coord.g.tests5')).toBe('item');
  });

  it('by thread follows the item and names the thread by its id', async () => {
    const board = { ...filterBoard(), threads: [threadDoc('lead', { session: 'Overlord' }), threadDoc('coder')] };
    const { container } = await mountTab(board);
    await screen.findByText('Login works');
    await pickView(/^Group/, 'Thread');
    expect(titlesIn(groupOf(container, 'coder'))).toEqual(['T-2']);
    expect(within(groupOf(container, 'lead')).getAllByText('lead').length).toBeGreaterThan(0);
  });

  it('returns to the runs and reads the saved choice on load', async () => {
    localStorage.setItem('coord.g.tests5', 'item');
    const { container } = await mountTab({ ...filterBoard(), items: [...filterBoard().items, testRunDoc('I-30', 'I-9')] });
    await screen.findByText('Login works');
    expect(groupOf(container, 'I-1')).not.toBeNull();
    await pickView(/^Group/, 'Run');
    expect(container.querySelector('[data-run="I-30"]')).not.toBeNull();
    expect(localStorage.getItem('coord.g.tests5')).toBe('run');
  });
});

describe('S1 the order', { tags: ['standard'] }, () => {
  const ranked = () => ({
    tests: [
      testDoc('T-1', { rank: 2, updatedAt: '2026-10-26T09:30:00.000Z' }),
      testDoc('T-2', { rank: 1, updatedAt: '2026-10-26T09:10:00.000Z' }),
    ],
  });
  const order = (container: HTMLElement): string[] => titlesIn(container);

  it('is the manual rank until the owner picks updated, and is saved with the filters', async () => {
    const { container } = await mountTab(ranked());
    await screen.findByText('Test number 1');
    expect(order(container)).toEqual(['T-2', 'T-1']);
    await pickView(/^Sort/, 'Updated');
    expect(order(container)).toEqual(['T-1', 'T-2']);
    expect(stored('coord.f')).toMatchObject({ sort: 'updated' });
  });
});

describe('S1 show passed', { tags: ['standard'] }, () => {
  const withPassed = () => ({ tests: [testDoc('T-1'), doneDoc('T-2', 'pass', 'fine'), doneDoc('T-3', 'fail', 'broken')] });

  it('is on by default, hides the passed tests when turned off and remembers it', async () => {
    await mountTab(withPassed());
    await screen.findByText('Test number 2');
    fireEvent.click(screen.getByRole('button', { name: 'Show passed' }));
    expect(screen.getByRole('button', { name: 'Show passed' }).getAttribute('aria-pressed')).toBe('false');
    expect(screen.queryByText('Test number 2')).toBeNull();
    expect(screen.getByText('Test number 3')).toBeTruthy();
    expect(localStorage.getItem('coord.passed')).toBe('0');
  });

  it('reads the saved choice on load', async () => {
    localStorage.setItem('coord.passed', '0');
    await mountTab(withPassed());
    await screen.findByText('Test number 3');
    expect(screen.queryByText('Test number 2')).toBeNull();
  });
});

describe('S1 closed runs and the search', { tags: ['standard'] }, () => {
  const closed = () => ({
    group: 'item',
    items: [itemDoc('I-1'), testRunDoc('I-40', 'I-9'), testRunDoc('I-41', 'I-9', { stage: 'passed' })],
    tests: [testDoc('T-1')],
  });

  it('are never reached by a search unless Show closed is on', async () => {
    const off = await mountTab(closed());
    await screen.findByText('Test number 1');
    await openFilters();
    typeSearch('I-41');
    expect(off.container.querySelector('[data-run="I-41"]')).toBeNull();
    off.unmount();
    localStorage.clear();
    localStorage.setItem('coord.closed', '1');
    const on = await mountTab(closed());
    await screen.findByText('Test number 1');
    await openFilters();
    typeSearch('I-41');
    expect(on.container.querySelector('[data-run="I-41"]')).not.toBeNull();
    expect(on.container.querySelector('[data-run="I-40"]')).toBeNull();
  });

  it('remember the Show closed choice under the key of the Node page', async () => {
    await mountTab(closed());
    fireEvent.click(await screen.findByRole('button', { name: 'Show closed (1)' }));
    expect(localStorage.getItem('coord.closed')).toBe('1');
    fireEvent.click(screen.getByRole('button', { name: 'Hide closed (1)' }));
    expect(localStorage.getItem('coord.closed')).toBe('0');
  });
});
