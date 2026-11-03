/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { buildDoc, itemDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';
import type { Doc } from './mount-tab';
import { outcomeButton, resultCalls } from './outcome-dom';

cleanAfterEach();

const planDoc = (id: string, cases: string[], extra: Doc = {}): Doc =>
  itemDoc(id, { title: `Plan number ${id.slice(2)}`, subject: null, type: 'test-plan', stage: 'ready', fields: { cases }, ...extra });

const board = (extra: Doc[] = []) => ({
  group: 'plan',
  builds: [buildDoc('bundle-1')],
  items: [itemDoc('I-1'), planDoc('I-5', ['T-1', 'T-2']), planDoc('I-6', ['T-2']), ...extra],
  tests: [testDoc('T-1'), testDoc('T-2'), testDoc('T-3')],
});

const idsIn = (container: HTMLElement, label: string): string[] =>
  [...(container.querySelector(`[role="list"][aria-label="${label}"]`)?.querySelectorAll('[data-test]') ?? [])].map((row) => row.getAttribute('data-test') ?? '');

const cardOf = (container: HTMLElement, code: string): HTMLElement => container.querySelector<HTMLElement>(`[data-plan="${code}"]`) as HTMLElement;

describe('P5 the plans as cards', { tags: ['standard'] }, () => {
  it('shows a card per plan with its code, title, progress and a Start run button', async () => {
    const { container } = await mountTab(board());
    await screen.findAllByText('Plan number 5');
    expect(container.querySelectorAll('[data-plan]')).toHaveLength(2);
    const card = within(cardOf(container, 'TP-5'));
    expect(card.getByText('Plan number 5')).toBeTruthy();
    expect(card.getByText('2 to run')).toBeTruthy();
    expect(card.getByRole('button', { name: 'Start run' })).toBeTruthy();
    expect(card.getByRole('button', { name: 'More actions for TP-5' })).toBeTruthy();
  });

  it('cannot start a run on a plan with no case', async () => {
    const { container } = await mountTab(board([planDoc('I-7', [])]));
    await screen.findAllByText('Plan number 7');
    expect(within(cardOf(container, 'TP-7')).getByRole('button', { name: 'Start run' }).hasAttribute('disabled')).toBe(true);
  });

  it('offers Continue run while a run is in progress', async () => {
    const live = planDoc('I-8', ['T-1'], { fields: { cases: ['T-1'], runs: [{ n: 1, bundle: null, by: 'human', startedAt: '2026-10-26T09:00:00.000Z', endedAt: null, results: [] }] } });
    const { container } = await mountTab(board([live]));
    await screen.findAllByText('Plan number 8');
    expect(within(cardOf(container, 'TP-8')).getByRole('button', { name: 'Continue run' })).toBeTruthy();
  });

  it('folds the passed plans under Closed plans', async () => {
    await mountTab(board([planDoc('I-9', ['T-1'], { stage: 'passed' })]));
    expect(await screen.findByText('Closed plans · 1')).toBeTruthy();
  });

  it('starts the run on the plan and opens its Run tab', async () => {
    const { container, calls } = await mountTab(board());
    await screen.findAllByText('Plan number 5');
    fireEvent.click(within(cardOf(container, 'TP-5')).getByRole('button', { name: 'Start run' }));
    const entries = () => calls.filter((call) => call.url.endsWith('/entries'));
    await waitFor(() => expect(entries()).toHaveLength(1));
    expect(entries()[0]?.body).toMatchObject({ kind: 'plan-run-start', target: { c: 'items', id: 'I-5' } });
    await waitFor(() => expect(new URLSearchParams(screen.getByTestId('search').textContent).get('pane')).toBe('run'));
    expect(new URLSearchParams(screen.getByTestId('search').textContent).get('open')).toBe('TP-5');
  });

  it('opens the plan pane from the title of the card', async () => {
    const { container } = await mountTab(board());
    await screen.findAllByRole('button', { name: 'Open TP-6: Plan number 6' });
    fireEvent.click(within(cardOf(container, 'TP-6')).getByRole('button', { name: 'Open TP-6: Plan number 6' }));
    await waitFor(() => expect(new URLSearchParams(screen.getByTestId('search').textContent).get('open')).toBe('TP-6'));
    expect(screen.getByTestId('hash').textContent).toBe('');
  });
});

describe('P5 the cases as rows grouped by plan', { tags: ['standard'] }, () => {
  it('lists a case under every plan that holds it, and the others under Not in a plan', async () => {
    const { container } = await mountTab(board());
    await screen.findAllByText('Plan number 5');
    expect(idsIn(container, 'Cases of TP-5')).toEqual(['T-1', 'T-2']);
    expect(idsIn(container, 'Cases of TP-6')).toEqual(['T-2']);
    expect(idsIn(container, 'Not in a plan')).toEqual(['T-3']);
  });

  it('opens the case pane over the tab when a row is clicked', async () => {
    await mountTab(board());
    const buttons = await screen.findAllByRole('button', { name: 'Open T-3: Test number 3' });
    fireEvent.click(buttons[0] as HTMLElement);
    await waitFor(() => expect(screen.getByTestId('search').textContent).toBe('?open=T-3'));
    expect(screen.getByTestId('hash').textContent).toBe('');
  });

  it('records an outcome from the buttons of the row', async () => {
    const { calls } = await mountTab(board());
    await screen.findByText('Test number 3');
    fireEvent.click(outcomeButton('pass', 'T-3'));
    await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
    expect(resultCalls(calls)[0]?.body).toEqual({ outcome: 'pass', text: '' });
  });

  it('puts a note box under the row for a fail', async () => {
    await mountTab(board());
    await screen.findByText('Test number 3');
    fireEvent.click(outcomeButton('fail', 'T-3'));
    expect(screen.getByRole('textbox', { name: 'Note for T-3' })).toBeTruthy();
  });

  it('groups by plan when nothing was saved, and saves the other choices under a new key', async () => {
    const { container } = await mountTab({ ...board(), group: 'default' });
    await screen.findAllByText('Plan number 5');
    expect(container.querySelector('[data-bundle]')).toBeNull();
    fireEvent.click(screen.getByRole('radio', { name: 'Bundle' }));
    expect(localStorage.getItem('coord.g.tests4')).toBe('build');
  });
});

describe('I-85 the plan card and the plan group header', { tags: ['standard'] }, () => {
  it('lets the progress bar of the plan card fill the width, not a fixed height', async () => {
    const { container } = await mountTab(board());
    await screen.findAllByText('Plan number 5');
    expect(cardOf(container, 'TP-5').querySelector('.bundle__bar')).toBeNull();
  });

  it('heads the cases of a plan with the type mark, the code link, the title and the count', async () => {
    const { container } = await mountTab(board());
    await screen.findAllByText('Plan number 5');
    const group = container.querySelector('[role="list"][aria-label="Cases of TP-5"]')?.closest('details') as HTMLElement;
    const head = within(group.querySelector('summary') as HTMLElement);
    expect(group.querySelector('summary [data-type-mark]')).not.toBeNull();
    expect(head.getByRole('link', { name: /TP-5/ })).toBeTruthy();
    expect(head.getByText('Plan number 5')).toBeTruthy();
    expect(group.querySelector('.disclosure__end')?.textContent).toBe('2');
    expect(group.querySelector('summary [data-record]')).toBeNull();
  });
});
