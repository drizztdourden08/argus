/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DEFAULT_FILTERS, filtersActive, planCases, planOptions } from '../../src/screens/board/tabs/tests/model';
import type { ItemDoc } from '../../src/screens/board/tabs/tests/model';
import { cleanAfterEach } from './board-dom';
import { openFilters, pickClause } from '../helpers/filter-clauses';
import { filterBoard, shown, stored } from './filter-dom';
import { itemDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const planDoc = (id: string, cases: string[], extra: object = {}) =>
  itemDoc(id, { title: `Plan ${id}`, subject: null, type: 'test-plan', fields: { cases }, ...extra });

const docs = (...list: object[]): ItemDoc[] => list as ItemDoc[];

const withPlans = () => {
  const board = filterBoard();
  return { ...board, items: [...board.items, planDoc('I-5', ['T-1', 'T-3']), planDoc('I-6', ['T-2'])] };
};

describe('I-70 P7 the plan filter model', { tags: ['critical'] }, () => {
  it('lists the live test plans by id, and nothing else', () => {
    const items = docs(planDoc('I-10', []), planDoc('I-9', []), planDoc('I-7', [], { deletedAt: 'x' }), itemDoc('I-1'));
    expect(planOptions(items)).toEqual([{ value: 'I-9', label: 'TP-9 Plan I-9' }, { value: 'I-10', label: 'TP-10 Plan I-10' }]);
  });

  it('reads the cases of the chosen plan, none for no plan, an empty set for an unknown one', () => {
    const items = docs(planDoc('I-5', ['T-1', 'T-3']), itemDoc('I-1', { fields: { cases: ['T-9'] } }));
    expect(planCases(items, '')).toBeNull();
    expect([...(planCases(items, 'I-5') ?? [])]).toEqual(['T-1', 'T-3']);
    expect(planCases(items, 'I-1')?.size).toBe(0);
    expect(planCases(items, 'I-404')?.size).toBe(0);
  });

  it('counts the plan as a filter in use', () => {
    expect(filtersActive({ ...DEFAULT_FILTERS, plan: 'I-5' })).toBe(true);
  });
});

describe('I-70 P7 the plan filter in the Tests tab', { tags: ['standard'] }, () => {
  it('has no Plan choice on a board without a plan', async () => {
    await mountTab(filterBoard());
    await screen.findByText('Login works');
    await openFilters();
    fireEvent.click(await screen.findByRole('button', { name: 'Add filter' }));
    expect(await screen.findByRole('menuitem', { name: 'Status' })).toBeTruthy();
    expect(screen.queryByRole('menuitem', { name: 'Plan' })).toBeNull();
  });

  it('narrows the tests to the cases of the plan, and removing the chip brings them back', async () => {
    await mountTab(withPlans());
    await screen.findByText('Login works');
    await openFilters();
    await pickClause('Plan', 'TP-5 Plan I-5');
    shown('T-1', 'T-3');
    expect(stored('coord.f')).toMatchObject({ plan: 'I-5' });
    await pickClause('Plan', 'TP-6 Plan I-6');
    shown('T-2');
    fireEvent.click(screen.getByRole('button', { name: 'Remove filter on Plan' }));
    shown('T-1', 'T-2', 'T-3');
  });

  it('reads the saved plan on load', async () => {
    localStorage.setItem('coord.f', JSON.stringify({ plan: 'I-6' }));
    await mountTab(withPlans());
    await screen.findByText('Logout works');
    shown('T-2');
  });
});
